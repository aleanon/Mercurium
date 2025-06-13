use deps::iced::{
    self,
    widget::{
        text_editor::{Action, Content, Edit},
        TextEditor,
    },
    Element,
};

#[derive(Debug, Clone)]
pub enum Message {
    Edit(Edit),
}

pub struct TextField {
    content: Content<iced::Renderer>,
    height: u32,
    max_height: Option<u32>,
    placeholder: &'static str,
}

impl TextField {
    pub fn new(placeholder: &'static str, height: u32, max_height: Option<u32>) -> Self {
        Self {
            content: Content::new(),
            height,
            max_height,
            placeholder,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Edit(edit) => {
                self.content.perform(Action::Edit(edit));
                let line_count = self.content.line_count();
                let height = line_count * 20;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let editor = TextEditor::new(&self.content)
            .placeholder(self.placeholder)
            .height(self.height);

        if let Some(max_height) = self.max_height {
            editor.max_height(max_height).into()
        } else {
            editor.into()
        }
    }
}
