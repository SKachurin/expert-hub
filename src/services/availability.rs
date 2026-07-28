use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc,
    Weekday,
};
use chrono_tz::Tz;
use serde::Serialize;
use serde_json::Value;
use sea_orm::{ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::{BTreeMap, HashSet};

use reqwest::StatusCode;
use crate::services::bookings::ACTIVE_BOOKING_BLOCKER_STATUSES;
use crate::services::booking_rules::PLATFORM_MIN_BOOKING_LEAD_MINUTES;

use crate::{
    config::AppConfig,
    entities::{bookings, calendar_connections},
    services::{
        calendar_connections::update_google_connection_tokens,
        experts::PublicExpertResponse,
        google_calendar::{
            fetch_google_free_busy_raw,
            parse_google_free_busy_response,
            refresh_google_access_token,
        },
    },
};

#[derive(Debug, Clone)]
struct Interval {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AvailabilitySlot {
    pub start_utc: String,
    pub start_local: String,
    pub end_local: String,
    pub duration_minutes: i64,
}

#[derive(Debug, Serialize)]
pub struct AvailabilityDay {
    pub date: String,
    pub label: String,
    pub slots: Vec<AvailabilitySlot>,
}

#[derive(Debug, Serialize)]
pub struct AvailabilityResponse {
    pub period_start: String,
    pub period_end: String,
    pub days: Vec<AvailabilityDay>,
}

pub async fn get_public_availability<C>(
    db: &C,
    config: &AppConfig,
    expert: &PublicExpertResponse,
    offset_days: i64,
    duration_minutes: Option<i64>,
) -> Result<AvailabilityResponse, String>
where
    C: ConnectionTrait,
{
    let tz: Tz = expert
        .timezone
        .parse()
        .map_err(|_| format!("invalid expert timezone: {}", expert.timezone))?;

    let now_utc = Utc::now();
    let today_local = now_utc.with_timezone(&tz).date_naive();

    let start_date = today_local + Duration::days(offset_days);
    let end_date = start_date + Duration::days(7);

    let period_start_local = local_datetime(tz, start_date, hm(0, 0)?)?;
    let period_end_local = local_datetime(tz, end_date, hm(0, 0)?)?;

    let period_start_utc = period_start_local.with_timezone(&Utc);
    let period_end_utc = period_end_local.with_timezone(&Utc);

    let mut response = AvailabilityResponse {
        period_start: start_date.format("%Y-%m-%d").to_string(),
        period_end: (end_date - Duration::days(1))
            .format("%Y-%m-%d")
            .to_string(),
        days: Vec::new(),
    };

    if !expert.is_active || !expert.is_bookable {
        return Ok(response);
    }

    let working_days = parse_working_days(&expert.working_days)?;
    let durations = parse_durations(&expert.allowed_session_durations)?;
    if durations.is_empty() {
      return Ok(response);
    }

    let default_duration = *durations.iter().min().unwrap();

    let selected_duration = match duration_minutes {
      Some(value) if durations.contains(&value) => value,
      Some(value) => {
          return Err(format!(
              "unsupported duration_minutes: {}. allowed: {:?}",
              value, durations
          ));
      }
      None => default_duration,
    };
    let work_start = NaiveTime::parse_from_str(&expert.work_start_time, "%H:%M")
        .map_err(|e| format!("invalid work_start_time: {e}"))?;
    let work_end = NaiveTime::parse_from_str(&expert.work_end_time, "%H:%M")
        .map_err(|e| format!("invalid work_end_time: {e}"))?;

    if work_end <= work_start {
        return Ok(response);
    }

    let latest_allowed_date = today_local + Duration::days(expert.max_days_ahead as i64);

    let effective_min_notice_minutes = (expert.minimum_notice_minutes as i64).max(PLATFORM_MIN_BOOKING_LEAD_MINUTES);

    let earliest_allowed_start = now_utc + Duration::minutes(effective_min_notice_minutes);

    let mut blocked = load_google_busy_intervals(
        db,
        config,
        expert.id,
        period_start_utc,
        period_end_utc,
        &expert.timezone,
    )
    .await?;

    blocked.extend(
        load_booking_busy_intervals(db, expert.id, period_start_utc, period_end_utc).await?
    );

    let blocked = merge_intervals(
        blocked
            .into_iter()
            .map(|iv| Interval {
                start: iv.start - Duration::minutes(expert.buffer_before_minutes as i64),
                end: iv.end + Duration::minutes(expert.buffer_after_minutes as i64),
            })
            .collect(),
    );

    let mut grouped: BTreeMap<NaiveDate, Vec<AvailabilitySlot>> = BTreeMap::new();

    let mut current_date = start_date;
    while current_date < end_date {
        if current_date <= latest_allowed_date && working_days.contains(weekday_code(current_date.weekday())) {
            let day_start_local = local_datetime(tz, current_date, work_start)?;
            let day_end_local = local_datetime(tz, current_date, work_end)?;

            let day_start_utc = day_start_local.with_timezone(&Utc);
            let day_end_utc = day_end_local.with_timezone(&Utc);

            let candidate_start = if day_start_utc > earliest_allowed_start {
                day_start_utc
            } else {
                earliest_allowed_start
            };

            if day_end_utc > candidate_start {
                let free_windows = subtract_intervals(
                    vec![Interval {
                        start: candidate_start,
                        end: day_end_utc,
                    }],
                    &blocked,
                );

                let mut day_slots = Vec::new();

                for free in free_windows {
                    let mut cursor = free.start;

                    while cursor + Duration::minutes(selected_duration) <= free.end {
                        let slot_end = cursor + Duration::minutes(selected_duration);
                        let local_start = cursor.with_timezone(&tz);
                        let local_end = slot_end.with_timezone(&tz);

                        day_slots.push(AvailabilitySlot {
                            start_utc: cursor.to_rfc3339(),
                            start_local: local_start.format("%H:%M").to_string(),
                            end_local: local_end.format("%H:%M").to_string(),
                            duration_minutes: selected_duration,
                        });

                        cursor += Duration::minutes(selected_duration);
                    }
                }

                day_slots.sort_by(|a, b| a.start_utc.cmp(&b.start_utc));

                if !day_slots.is_empty() {
                    grouped.insert(current_date, day_slots);
                }
            }
        }

        current_date += Duration::days(1);
    }

    response.days = grouped
        .into_iter()
        .map(|(date, slots)| AvailabilityDay {
            date: date.format("%Y-%m-%d").to_string(),
            label: date.format("%a, %-d %b").to_string(),
            slots,
        })
        .collect();

    Ok(response)
}

async fn load_google_busy_intervals<C>(
    db: &C,
    config: &AppConfig,
    expert_id: i64,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    time_zone: &str,
) -> Result<Vec<Interval>, String>
where
    C: ConnectionTrait,
{
    let rows = calendar_connections::Entity::find()
        .filter(calendar_connections::Column::ExpertId.eq(expert_id))
        .filter(calendar_connections::Column::Provider.eq("google"))
        .filter(calendar_connections::Column::IsEnabled.eq(true))
        .all(db)
        .await
        .map_err(|e| format!("failed to load calendar connections: {e}"))?;

    let mut intervals = Vec::new();

    for row in rows {
        let Some(calendar_id) = row.selected_calendar_id.clone().filter(|v| !v.trim().is_empty()) else {
            continue;
        };

        let Some(access_token) = row.access_token.clone().filter(|v| !v.trim().is_empty()) else {
            continue;
        };

        let mut final_access_token = access_token;

        let first_resp = fetch_google_free_busy_raw(
            &final_access_token,
            &[calendar_id.clone()],
            period_start_utc,
            period_end_utc,
            time_zone,
        )
        .await?;

        let resp = if first_resp.status() == StatusCode::UNAUTHORIZED {
            let Some(refresh_token) = row.refresh_token.clone().filter(|v| !v.trim().is_empty()) else {
                return Err(format!(
                    "google connection {} needs reconnect: missing refresh token",
                    row.id
                ));
            };

            let (new_access_token, new_expires_at) = refresh_google_access_token(
                &config.google_client_id,
                &config.google_client_secret,
                &refresh_token,
            )
            .await?;

            update_google_connection_tokens(
                db,
                row.id,
                new_access_token.clone(),
                new_expires_at,
            )
            .await?;

            final_access_token = new_access_token;

            fetch_google_free_busy_raw(
                &final_access_token,
                &[calendar_id.clone()],
                period_start_utc,
                period_end_utc,
                time_zone,
            )
            .await?
        } else {
            first_resp
        };

        let fetched = parse_google_free_busy_response(resp).await?;

        intervals.extend(
            fetched.into_iter().map(|v| Interval {
                start: v.start,
                end: v.end,
            })
        );
    }

    Ok(intervals)
}

async fn load_booking_busy_intervals<C>(
    db: &C,
    expert_id: i64,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
) -> Result<Vec<Interval>, String>
where
    C: ConnectionTrait,
{
    let rows = bookings::Entity::find()
        .filter(bookings::Column::ExpertId.eq(expert_id))
        .filter(bookings::Column::SlotStart.lt(period_end_utc.fixed_offset()))
        .filter(bookings::Column::SlotEnd.gt(period_start_utc.fixed_offset()))
        .filter(
            Condition::any()
                .add(
                    bookings::Column::Status.is_in(
                        ACTIVE_BOOKING_BLOCKER_STATUSES.iter().copied()
                    )
                )
                .add(bookings::Column::ExpertRejectedAt.is_not_null())
        )
        .all(db)
        .await
        .map_err(|e| format!("failed to load bookings: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|row| Interval {
            start: row.slot_start.with_timezone(&Utc),
            end: row.slot_end.with_timezone(&Utc),
        })
        .collect())
}

fn parse_working_days(value: &Value) -> Result<HashSet<String>, String> {
    let arr = value
        .as_array()
        .ok_or_else(|| "working_days must be an array".to_string())?;

    Ok(arr
        .iter()
        .filter_map(|v| v.as_str())
        .map(|v| v.trim().to_lowercase())
        .collect())
}

fn parse_durations(value: &Value) -> Result<Vec<i64>, String> {
    let arr = value
        .as_array()
        .ok_or_else(|| "allowed_session_durations must be an array".to_string())?;

    let mut durations: Vec<i64> = arr
        .iter()
        .filter_map(|v| v.as_i64())
        .filter(|v| *v > 0)
        .collect();

    durations.sort_unstable();
    durations.dedup();

    Ok(durations)
}

fn weekday_code(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "mon",
        Weekday::Tue => "tue",
        Weekday::Wed => "wed",
        Weekday::Thu => "thu",
        Weekday::Fri => "fri",
        Weekday::Sat => "sat",
        Weekday::Sun => "sun",
    }
}

fn local_datetime(tz: Tz, date: NaiveDate, time: NaiveTime) -> Result<DateTime<Tz>, String> {
    let naive = NaiveDateTime::new(date, time);

    match tz.from_local_datetime(&naive) {
        LocalResult::Single(dt) => Ok(dt),
        LocalResult::Ambiguous(first, _) => Ok(first),
        LocalResult::None => Err(format!("invalid local datetime for timezone {}", tz)),
    }
}

fn hm(hour: u32, minute: u32) -> Result<NaiveTime, String> {
    NaiveTime::from_hms_opt(hour, minute, 0)
        .ok_or_else(|| "invalid time".to_string())
}

fn merge_intervals(mut intervals: Vec<Interval>) -> Vec<Interval> {
    if intervals.is_empty() {
        return intervals;
    }

    intervals.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));

    let mut merged = Vec::new();
    let mut current = intervals[0].clone();

    for next in intervals.into_iter().skip(1) {
        if next.start <= current.end {
            if next.end > current.end {
                current.end = next.end;
            }
        } else {
            merged.push(current);
            current = next;
        }
    }

    merged.push(current);
    merged
}

fn subtract_intervals(free: Vec<Interval>, blocked: &[Interval]) -> Vec<Interval> {
    let mut result = Vec::new();

    for free_iv in free {
        let mut cursor = free_iv.start;

        for block in blocked {
            if block.end <= cursor {
                continue;
            }

            if block.start >= free_iv.end {
                break;
            }

            if block.start > cursor {
                result.push(Interval {
                    start: cursor,
                    end: block.start.min(free_iv.end),
                });
            }

            if block.end >= free_iv.end {
                cursor = free_iv.end;
                break;
            }

            if block.end > cursor {
                cursor = block.end;
            }
        }

        if cursor < free_iv.end {
            result.push(Interval {
                start: cursor,
                end: free_iv.end,
            });
        }
    }

    result
}