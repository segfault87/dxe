use std::error::Error;

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait EventStateCallback<T> {
    fn on_initialized(&self) {}

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

#[async_trait::async_trait]
pub trait PresenceCallback {
    async fn on_enter(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn on_leave(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
