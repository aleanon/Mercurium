use std::i16::MAX;

use deps::iced::{
    self,
    advanced::{graphics::core::SmolStr, text::highlighter::PlainText},
    event::Status,
    keyboard::{key::Named, Key, Modifiers},
    widget::{
        text_editor::{self, Action, Binding, Content, Edit, KeyPress, Motion},
        TextEditor,
    },
    Element, Point, Task, Theme,
};

#[derive(Debug, Clone)]
pub enum Message {
    Edit(Edit),
    Drag(Point),
    Click(Point),
    Move(Motion),
    Scroll(i32),
    SelectWord,
    SelectAll,
    SelectLine,
    Select(Motion),
    None,
}

#[derive(Debug)]
pub struct TextField {
    content: Content<iced::Renderer>,
}

impl TextField {
    pub fn new() -> Self {
        Self {
            content: Content::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(edit) => self.content.perform(Action::Edit(edit)),
            Message::SelectWord => self.content.perform(Action::SelectWord),
            Message::SelectLine => self.content.perform(Action::SelectLine),
            Message::SelectAll => self.content.perform(Action::SelectAll),
            Message::Select(motion) => self.content.perform(Action::Select(motion)),
            Message::Click(point) => self.content.perform(Action::Click(point)),
            Message::Drag(point) => self.content.perform(Action::Drag(point)),
            Message::Move(motion) => self.content.perform(Action::Move(motion)),
            Message::Scroll(lines) => self.content.perform(Action::Scroll { lines }),
            Message::None => {}
        }
        Task::none()
    }

    pub fn view<M>(&self) -> TextEditor<'_, PlainText, Message, Theme, iced::Renderer> {
        TextEditor::new(&self.content)
            .key_binding(|key_press| match key_press {
                KeyPress {
                    key: Key::Named(Named::Backspace),
                    modifiers: Modifiers::CTRL,
                    text: None,
                    status: text_editor::Status::Focused { is_hovered: false },
                } => Some(Binding::Sequence(vec![
                    Binding::Select(Motion::WordLeft),
                    Binding::Backspace,
                ])),
                _ => None,
            })
            .on_action(|action| match action {
                Action::Edit(edit) => Message::Edit(edit),
                Action::Click(point) => Message::Click(point),
                Action::Drag(point) => Message::Drag(point),
                Action::Move(motion) => Message::Move(motion),
                Action::Scroll { lines } => Message::Scroll(lines),
                Action::SelectWord => Message::SelectWord,
                Action::SelectLine => Message::SelectLine,
                Action::SelectAll => Message::SelectAll,
                Action::Select(motion) => Message::Select(motion),
            })
    }
}
