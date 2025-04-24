
use deps::*;

use thiserror::Error;
use tokio::{sync::Mutex, task::JoinHandle};


#[derive(Debug, Clone, Error)]
pub enum TaskError {
    #[error("Task was never started")]
    TaskNotStarted,
    #[error("Failed to join task")]
    FailedToJoinTask,
    #[error("Task failed to complete")]
    TaskFailed,
    #[error("Result already claimed")]
    ResultTaken
}


 pub struct Task<T, E>(Mutex<TaskInner<T, E>>) where 
    T: Clone + Send + 'static,
    E: std::error::Error + Send + 'static + From<TaskError>,
;

impl<T, E> Task<T, E> 
    where 
        T: Clone + Send + 'static,
        E: std::error::Error + Send + 'static + From<TaskError>,
{
    pub fn new() -> Self {
        Self(Mutex::new(TaskInner::none()))
    }

    pub async fn run_task<F, Output>(&self, task_id: u16, task: F)  
        where
            F: FnOnce() -> Output + Send + 'static,
            E: std::error::Error + Send + 'static,
            Output: Future<Output = Result<T,E>> + Send + 'static,
        {
        self.0.lock().await.run(task_id, task);
    }

    pub async fn get_result(&self) -> Result<T, E> {
        let mut runner_inner = self.0.lock().await;
        runner_inner.get_result().await
    }
}

#[derive(Default)]
pub enum TaskState<T, E>  {
    #[default]
    NotStarted,
    Running(JoinHandle<Result<T, E>>),
    Complete(T),
    Failed,
}


struct TaskInner<T, E: std::error::Error> {
    current_task_id: u16,
    state: TaskState<T, E>,
}

impl<T, E> TaskInner<T, E> where 
    T: Clone + Send + 'static,
    E: std::error::Error + Send + 'static + From<TaskError>,
{

    fn none() -> Self {
        Self {
            current_task_id: 0,
            state: TaskState::NotStarted,
        }
    }

    fn run<F, Output>(&mut self, task_id: u16, task: F) 
        where 
            F: FnOnce() -> Output + Send + 'static,
            Output: Future<Output = Result<T, E>> + Send + 'static,
        {
        if self.current_task_id >=  task_id {
            return
        }

        if let TaskState::Running(handle) = &mut self.state {
            handle.abort();
        }

        self.state = TaskState::Running(tokio::spawn(task()));
    }

    async fn get_result(&mut self) -> Result<T, E> {
        match &mut self.state {
            TaskState::NotStarted => return Err(TaskError::TaskNotStarted.into()),
            TaskState::Running(handle) => {
                match handle.await {
                    Ok(result) => {
                        match result {
                            Ok(value) => {
                                self.state = TaskState::Complete(value.clone());
                                return Ok(value)
                            }
                            Err(err) => {
                                self.state = TaskState::Failed;
                                return Err(err)
                            }
                        }
                    }
                    Err(_) => {
                        self.state = TaskState::Failed;
                        return Err(TaskError::FailedToJoinTask.into())
                    }
                }
            }
            TaskState::Complete(value) => return Ok(value.clone()),
            TaskState::Failed => return Err(TaskError::TaskFailed.into()),
        }
    }

    pub fn into_task_state(&mut self) -> TaskState<T,E> {
        std::mem::replace(&mut self.state, TaskState::NotStarted)
    }

 }