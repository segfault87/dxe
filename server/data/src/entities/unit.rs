use sqlx::FromRow;

use crate::types::UnitId;

#[derive(Debug, Clone, FromRow)]
pub struct Unit {
    pub id: UnitId,
    pub enabled: bool,
}
