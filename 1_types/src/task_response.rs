#[derive(Debug, Clone)]
pub struct TaskResponse<T> {
    task_id: u8,
    data: Option<T>
}

impl<T> TaskResponse<T> {
    pub fn new(task_id: u8) -> Self {
        Self { task_id, data: None }
    }

    pub fn new_with_data(task_id: u8, data: T) -> Self {
        Self { task_id, data: Some(data) }
    }

    pub fn new_response(&mut self, new_response: TaskResponse<T>) {
        if self.task_id <= new_response.task_id {*self = new_response}
    }

    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub fn take_data(&mut self) -> Option<T> {
        self.data.take()
    }

    pub fn discard_data(&mut self) {
        self.data = None;
    }

    pub fn new_task_id(&mut self) -> u8 {
        self.task_id += 1;
        self.task_id
    }
}