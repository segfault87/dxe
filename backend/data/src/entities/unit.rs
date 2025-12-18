use dxe_types::{SpaceId, UnitId};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Space {
    pub id: SpaceId,
    pub enabled: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct Unit {
    pub id: UnitId,
    pub space_id: SpaceId,
    pub enabled: bool,
}
