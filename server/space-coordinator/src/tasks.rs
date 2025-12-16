pub mod audio_recorder;
pub mod booking_state_manager;
pub mod carpark_exempter;
pub mod osd_controller;
pub mod presence_monitor;
pub mod telemetry_manager;
pub mod z2m_controller;

use std::collections::HashMap;

use tokio_task_scheduler::{Scheduler, Task};

pub struct TaskContext {
    pub scheduler: Scheduler,

    tasks: HashMap<String, String>,
}

impl TaskContext {
    pub async fn new() -> Result<Self, Error> {
        let scheduler = Scheduler::new();

        Ok(Self {
            scheduler,

            tasks: HashMap::new(),
        })
    }

    pub async fn add_task(&mut self, task: Task) -> Result<(), Error> {
        let task_name = task.get_name().to_owned();
        let id = self.scheduler.add_task(task).await?;

        log::info!("Task {task_name} added. id: {id}");

        self.tasks.insert(task_name, id);

        Ok(())
    }

    pub async fn run(&self) {
        let mut stop_receiver = self.scheduler.start().await;

        let _ = stop_receiver.recv().await;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't add task: {0}")]
    Scheduler(#[from] tokio_task_scheduler::SchedulerError),
}
