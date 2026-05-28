use std::error::Error;
use std::sync::Arc;

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait LifecycleEventCallback<T> {
    async fn on_start(self: Arc<Self>, event: &T) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn on_end(self: Arc<Self>, event: &T) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
