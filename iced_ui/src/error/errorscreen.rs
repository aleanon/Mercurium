use iced::Task;
use types::AppError;

use crate::{
    app::{AppMessage, AppState},
    App,
};

#[derive(Debug, Clone)]
pub enum ErrorMessage {
    
}

pub struct ErrorScreen {
    error: AppError,
}

impl<'a> ErrorMessage {
    pub fn update(self, app: &'a mut App) -> Task<AppMessage> {
        Task::none()
    }

}
