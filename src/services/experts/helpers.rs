use chrono::NaiveTime;
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::entities::experts;

pub fn format_time(value: &NaiveTime) -> String {
    value.format("%H:%M").to_string()
}

pub fn build_display_name(first_name: &str, last_name: &str, username: &str) -> String {
    let full = [first_name.trim(), last_name.trim()]
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if !full.is_empty() {
        return full;
    }

    if !username.trim().is_empty() {
        return username.trim().to_string();
    }

    "Telegram user".to_string()
}

pub fn parse_time(value: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(value, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M"))
        .map_err(|_| format!("invalid time format: {value}"))
}

pub fn optional_trimmed_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn slugify_part(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in input.trim().to_lowercase().chars() {
        let mapped = match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            '_' | ' ' | '-' => Some('-'),
            _ => None,
        };

        if let Some(c) = mapped {
            if c == '-' {
                if !prev_dash && !out.is_empty() {
                    out.push('-');
                    prev_dash = true;
                }
            } else {
                out.push(c);
                prev_dash = false;
            }
        }
    }

    out.trim_matches('-').to_string()
}

pub fn short_suffix() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

pub async fn generate_unique_public_slug<C: ConnectionTrait>(
    db: &C,
    username: &str,
    display_name: &str,
) -> Result<String, String> {
    let mut base = if !username.trim().is_empty() {
        slugify_part(username)
    } else {
        slugify_part(display_name)
    };

    if base.is_empty() {
        base = "expert".to_string();
    }

    let exists = experts::Entity::find()
        .filter(experts::Column::PublicSlug.eq(base.clone()))
        .one(db)
        .await
        .map_err(|e| format!("failed to check public slug: {e}"))?;

    if exists.is_none() {
        return Ok(base);
    }

    for _ in 0..20 {
        let candidate = format!("{}-{}", base, short_suffix());

        let exists = experts::Entity::find()
            .filter(experts::Column::PublicSlug.eq(candidate.clone()))
            .one(db)
            .await
            .map_err(|e| format!("failed to check public slug: {e}"))?;

        if exists.is_none() {
            return Ok(candidate);
        }
    }

    Err("failed to generate unique public slug".to_string())
}