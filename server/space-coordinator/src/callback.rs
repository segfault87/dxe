use std::error::Error;

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait EventStateCallback<T> {
    async fn on_event_created(
        &self,
        event: &T,
        is_in_progress: bool,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn on_event_deleted(
        &self,
        event: &T,
        is_in_progress: bool,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn on_event_start(&self, event: &T, buffered: bool) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn on_event_end(&self, event: &T, buffered: bool) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
