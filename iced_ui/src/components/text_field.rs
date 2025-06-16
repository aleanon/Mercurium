use deps::iced::{
    self,
    advanced::text::highlighter::PlainText,
    keyboard::{key::Named, Key, Modifiers},
    widget::{
        text_editor::{self, Action, Binding, Content, Edit, KeyPress, Motion},
        TextEditor,
    },
    Point, Task, Theme,
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
        }
        Task::none()
    }

    pub fn view<M>(
        &self,
        map_message: fn(Message) -> M,
    ) -> TextEditor<'_, PlainText, M, Theme, iced::Renderer>
    where
        // F: Fn(Message) -> M + 'static,
        M: Clone + 'static,
    {
        TextEditor::new(&self.content)
            .key_binding(Self::key_bindings)
            .on_action(move |action| match action {
                Action::Edit(edit) => map_message(Message::Edit(edit)),
                Action::Click(point) => map_message(Message::Click(point)),
                Action::Drag(point) => map_message(Message::Drag(point)),
                Action::Move(motion) => map_message(Message::Move(motion)),
                Action::Scroll { lines } => map_message(Message::Scroll(lines)),
                Action::SelectWord => map_message(Message::SelectWord),
                Action::SelectLine => map_message(Message::SelectLine),
                Action::SelectAll => map_message(Message::SelectAll),
                Action::Select(motion) => map_message(Message::Select(motion)),
            })
    }

    fn key_bindings<M: Clone + 'static>(key_press: KeyPress) -> Option<Binding<M>> {
        match key_press {
            KeyPress {
                key: Key::Named(Named::Backspace),
                modifiers: Modifiers::COMMAND,
                text: None,
                status:
                    text_editor::Status::Focused {
                        is_hovered: true | false,
                    },
            } => Some(Binding::Sequence(vec![
                Binding::Select(Motion::WordLeft),
                Binding::Backspace,
            ])),
            KeyPress {
                key: Key::Character(ref c),
                modifiers: Modifiers::COMMAND,
                text: None,
                status:
                    text_editor::Status::Focused {
                        is_hovered: true | false,
                    },
            } if c.as_str() == "l" => Some(Binding::SelectLine),
            _ => Binding::from_key_press(key_press),
        }
    }
}
