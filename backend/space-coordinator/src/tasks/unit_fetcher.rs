use std::collections::HashSet;
use std::sync::Arc;

use dxe_s2s_shared::handlers::GetUnitsResponse;
use dxe_types::UnitId;
use parking_lot::Mutex;
use tokio_task_scheduler::{Task, TaskBuilder};

use crate::client::{DxeClient, Error as ClientError};

#[derive(Clone, Debug)]
pub struct UnitsState(Arc<Mutex<HashSet<UnitId>>>);

impl UnitsState {
    pub fn new(initial: impl Iterator<Item = UnitId>) -> Self {
        Self(Arc::new(Mutex::new(initial.collect())))
    }

    pub fn get(&self) -> HashSet<UnitId> {
        self.0.lock().clone()
    }
}

pub struct UnitFetcher {
    client: DxeClient,

    state: UnitsState,
}

impl UnitFetcher {
    pub async fn new(client: DxeClient) -> Result<Self, ClientError> {
        let result = client.get::<GetUnitsResponse>("/units", None).await?;

        let state = UnitsState::new(result.units.into_iter().map(|v| v.id));

        Ok(Self { client, state })
    }

    pub fn state(&self) -> UnitsState {
        self.state.clone()
    }

    async fn update(self: Arc<Self>) {
        if let Ok(result) = self.client.get::<GetUnitsResponse>("/units", None).await {
            *self.state.0.lock() = result.units.iter().map(|v| v.id.clone()).collect();
        }
    }

    pub fn task(self) -> (Arc<Self>, Task) {
        let task_name = "unit_fetcher".to_string();

        let arc_self = Arc::new(self);

        (
            arc_self.clone(),
            TaskBuilder::new(&task_name, move || {
                let arc_self = arc_self.clone();
                tokio::task::spawn(async move {
                    arc_self.update().await;
                });

                Ok(())
            })
            .daily()
            .build(),
        )
    }
}
