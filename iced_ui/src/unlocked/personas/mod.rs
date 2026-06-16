use deps::*;

use iced::{
    Alignment, Element, Length, Padding, Task,
    widget::{self, button, column, container, row, text, text_input},
};
use types::{AppError, Notification, Persona, crypto::Password};
use wallet::{Unlocked, Wallet};
use zeroize::Zeroize;

use crate::{app::AppMessage, styles, unlocked::app_view};

#[derive(Debug, Clone)]
pub enum Message {
    InputLabel(String),
    InputPassword(String),
    Create,
    /// The async persona creation finished; carries the new persona on success.
    Created(Result<Persona, AppError>),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::PersonasViewMessage(self))
    }
}

#[derive(Debug)]
pub struct PersonasView {
    label: String,
    password: Password,
}

impl<'a> PersonasView {
    pub fn new() -> Self {
        Self {
            label: String::new(),
            password: Password::new(),
        }
    }

    pub fn update(&mut self, message: Message, wallet: &mut Wallet<Unlocked>) -> Task<AppMessage> {
        match message {
            Message::InputLabel(input) => self.label = input,
            Message::InputPassword(mut input) => {
                self.password.clear();
                self.password.push_str(input.as_str());
                input.zeroize();
            }
            Message::Create => {
                if self.label.is_empty() || self.password.is_empty() {
                    return Task::done(AppMessage::Error(AppError::NonFatal(Notification::Info(
                        "Enter a label and your password".to_string(),
                    ))));
                }
                let label = std::mem::take(&mut self.label);
                let password = std::mem::replace(&mut self.password, Password::new());
                let handle = wallet.create_persona_handle(label, password);
                return Task::perform(async move { handle.await }, |joined| {
                    match joined {
                        Ok(result) => Message::Created(result).into(),
                        Err(join_err) => AppMessage::Error(AppError::NonFatal(Notification::Info(
                            format!("Persona task failed: {join_err}"),
                        ))),
                    }
                });
            }
            Message::Created(result) => match result {
                Ok(persona) => {
                    let address = persona.identity_address.clone();
                    wallet.register_persona(persona);
                    return Task::done(AppMessage::Error(AppError::NonFatal(Notification::Info(
                        format!("Persona created: {address}"),
                    ))));
                }
                Err(err) => return Task::done(AppMessage::Error(err)),
            },
        }
        Task::none()
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        let header = text("Personas").size(20).width(Length::Fill).center();

        let mut personas: Vec<&Persona> = wallet.personas().values().collect();
        personas.sort_unstable_by(|a, b| a.id.cmp(&b.id));

        let list = if personas.is_empty() {
            column![text("No personas yet").style(styles::text::muted)]
        } else {
            column(personas.into_iter().map(|persona| {
                container(
                    column![
                        text(persona.label.clone()).size(15),
                        text(persona.identity_address.clone())
                            .size(11)
                            .style(styles::text::muted),
                    ]
                    .spacing(2),
                )
                .padding(10)
                .width(Length::Fill)
                .style(styles::container::weak_layer_2_rounded_with_shadow)
                .into()
            }))
            .spacing(10)
        };

        let label_input = text_input("Persona label", &self.label)
            .padding(10)
            .style(styles::text_input::base_layer_1_rounded)
            .on_input(|input| Message::InputLabel(input).into());

        let password_input = text_input("Password", self.password.as_str())
            .secure(true)
            .padding(10)
            .style(styles::text_input::base_layer_1_rounded)
            .on_input(|input| Message::InputPassword(input).into())
            .on_submit(Message::Create.into());

        let create_enabled = !self.label.is_empty() && !self.password.is_empty();
        let create_button = button(text("Create persona").center().width(Length::Fill))
            .width(Length::Fill)
            .height(45)
            .style(styles::button::primary)
            .on_press_maybe(create_enabled.then(|| Message::Create.into()));

        let create_form = container(
            column![
                text("New persona").size(15),
                label_input,
                password_input,
                create_button
            ]
            .spacing(10),
        )
        .padding(15)
        .style(styles::container::weak_layer_2_rounded_with_shadow);

        let content = column![header, list, create_form]
            .spacing(20)
            .align_x(Alignment::Center)
            .padding(Padding {
                left: 10.,
                right: 15.,
                top: 10.,
                bottom: 10.,
            });

        widget::scrollable(content)
            .style(styles::scrollable::vertical_scrollable_secondary)
            .into()
    }
}
