use deps::iced::{
    self, Point, Theme,
    advanced::text::highlighter::PlainText,
    keyboard::{
        Key,
        key::{Code, Physical},
    },
    widget::{
        TextEditor,
        text_editor::{Action, Binding, Content, Edit, KeyPress, Motion},
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    PerformAction(Action),
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

    pub fn update(&mut self, message: Message) {
        match message {
            Message::PerformAction(action) => self.content.perform(action),
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
    }

    pub fn view<F, M>(&self, map: F) -> TextEditor<'_, PlainText, M, Theme, iced::Renderer>
    where
        F: Fn(Message) -> M + 'static,
        M: Clone + 'static,
    {
        TextEditor::new(&self.content)
            .key_binding(Self::key_bindings)
            .on_action(move |action| map(Message::PerformAction(action)))
    }

    fn key_bindings<M: Clone + 'static>(key_press: KeyPress) -> Option<Binding<M>> {
        // Needs fixing, custom bindings don't work
        match key_press.physical_key {
            Physical::Code(Code::Backspace) if key_press.modifiers.command() => {
                return Some(Binding::Sequence(vec![
                    Binding::Select(Motion::WordLeft),
                    Binding::Backspace,
                ]));
            }
            _ => {}
        };

        match key_press.key.as_ref() {
            Key::Character("l") if key_press.modifiers.command() => {
                return Some(Binding::SelectLine);
            }
            _ => {}
        };
        Binding::from_key_press(key_press)
    }
}
