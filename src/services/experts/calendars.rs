use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::entities::calendar_connections;

use super::dto::EditCalendarOption;

pub fn build_calendar_label(model: &calendar_connections::Model) -> String {
    if let Some(label) = &model.connection_label {
        if !label.trim().is_empty() {
            return label.clone();
        }
    }

    match (
        model.provider.as_str(),
        model.account_email.as_deref(),
        model.selected_calendar_name.as_deref(),
    ) {
        ("google", Some(email), Some(name)) => format!("Google · {} · {}", email, name),
        ("google", Some(email), None) => format!("Google · {}", email),
        ("google", None, Some(name)) => format!("Google · {}", name),
        _ => model.provider.clone(),
    }
}

pub async fn load_edit_calendar_options<C: ConnectionTrait>(
    db: &C,
    expert_id: i64,
) -> Result<Vec<EditCalendarOption>, String> {
    let rows = calendar_connections::Entity::find()
        .filter(calendar_connections::Column::ExpertId.eq(expert_id))
        .order_by_desc(calendar_connections::Column::IsPrimary)
        .order_by_asc(calendar_connections::Column::Id)
        .all(db)
        .await
        .map_err(|e| format!("failed to load calendar connections: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|row| EditCalendarOption {
            id: row.id,
            provider: row.provider.clone(),
            connection_label: build_calendar_label(&row),
            is_primary: row.is_primary,
            is_enabled: row.is_enabled,
            selected_calendar_name: row.selected_calendar_name.clone(),
            selected_calendar_timezone: row.selected_calendar_timezone.clone(),
        })
        .collect())
}