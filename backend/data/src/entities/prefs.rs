use chrono::{DateTime, Utc};
use dxe_types::entities::MixerPreferences;
use dxe_types::{IdentityId, UnitId};
use sqlx::FromRow;
use sqlx::types::Json;

#[derive(Debug, Clone, FromRow)]
pub struct MixerConfig {
    pub identity_id: IdentityId,
    pub unit_id: UnitId,
    pub data: Json<MixerPreferences>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
