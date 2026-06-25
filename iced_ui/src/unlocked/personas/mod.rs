use deps::*;

use iced::{
    Alignment, Element, Length, Padding, Task,
    widget::{self, button, column, container, row, text, text_input},
};
use types::{
    AppError, NameVariant, Notification, Persona, PersonaData, PersonaName,
    crypto::Password,
};
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
    // --- Persona-data editing ---
    /// Begin editing the persona with this identity address, loading its current data.
    EditPersona(String),
    EditGiven(String),
    EditFamily(String),
    EditEmail(String),
    EditPhone(String),
    SaveData,
    CancelEdit,
}

/// In-progress edit of one persona's shared data. A single email/phone are exposed here for
/// simplicity; the underlying model holds vectors, so a non-empty value maps to a one-element list.
#[derive(Debug, Clone, Default)]
struct PersonaEdit {
    identity_address: String,
    given: String,
    family: String,
    email: String,
    phone: String,
}

impl PersonaEdit {
    fn from_persona(persona: &Persona) -> Self {
        let data = &persona.persona_data;
        let (given, family) = data
            .name
            .as_ref()
            .map(|n| (n.given.clone(), n.family.clone()))
            .unwrap_or_default();
        Self {
            identity_address: persona.identity_address.clone(),
            given,
            family,
            email: data.email_addresses.first().cloned().unwrap_or_default(),
            phone: data.phone_numbers.first().cloned().unwrap_or_default(),
        }
    }

    /// Maps the form back into the `PersonaData` model, dropping empty fields.
    fn to_persona_data(&self) -> PersonaData {
        let name = if self.given.is_empty() && self.family.is_empty() {
            None
        } else {
            Some(PersonaName {
                given: self.given.clone(),
                family: self.family.clone(),
                variant: NameVariant::default(),
            })
        };
        let collect = |v: &str| {
            if v.is_empty() {
                Vec::new()
            } else {
                vec![v.to_string()]
            }
        };
        PersonaData {
            name,
            email_addresses: collect(&self.email),
            phone_numbers: collect(&self.phone),
        }
    }
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
    editing: Option<PersonaEdit>,
}

impl<'a> PersonasView {
    pub fn new() -> Self {
        Self {
            label: String::new(),
            password: Password::new(),
            editing: None,
        }
    }

    pub fn update(&mut self, message: Message, wallet: &mut Wallet<Unlocked>) -> Task<AppMessage> {
        match message {
            Message::InputLabel(input) => self.label = input,
            Message::EditPersona(identity_address) => {
                self.editing = wallet
                    .personas()
                    .get(&identity_address)
                    .map(PersonaEdit::from_persona);
            }
            Message::EditGiven(input) => {
                if let Some(edit) = &mut self.editing {
                    edit.given = input;
                }
            }
            Message::EditFamily(input) => {
                if let Some(edit) = &mut self.editing {
                    edit.family = input;
                }
            }
            Message::EditEmail(input) => {
                if let Some(edit) = &mut self.editing {
                    edit.email = input;
                }
            }
            Message::EditPhone(input) => {
                if let Some(edit) = &mut self.editing {
                    edit.phone = input;
                }
            }
            Message::CancelEdit => self.editing = None,
            Message::SaveData => {
                if let Some(edit) = self.editing.take() {
                    if let Err(err) =
                        wallet.update_persona_data(&edit.identity_address, edit.to_persona_data())
                    {
                        return Task::done(AppMessage::Error(err));
                    }
                    return Task::done(AppMessage::Error(AppError::NonFatal(Notification::Info(
                        "Persona data saved".to_string(),
                    ))));
                }
            }
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
            column(personas.into_iter().map(|persona| self.persona_card(persona)))
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

    /// One persona row: label + address, the shared data (if any), and either an "Edit data"
    /// button or — when this persona is being edited — the inline edit form.
    fn persona_card(&'a self, persona: &'a Persona) -> Element<'a, AppMessage> {
        let mut card = column![
            text(persona.label.clone()).size(15),
            text(persona.identity_address.clone())
                .size(11)
                .style(styles::text::muted),
        ]
        .spacing(2);

        let is_editing = self
            .editing
            .as_ref()
            .is_some_and(|edit| edit.identity_address == persona.identity_address);

        if is_editing {
            card = card.push(self.edit_form());
        } else {
            // Read-only display of the currently shared data, if any.
            let data = &persona.persona_data;
            if let Some(name) = &data.name {
                card = card.push(text(format!("Name: {} {}", name.given, name.family)).size(12));
            }
            for email in &data.email_addresses {
                card = card.push(text(format!("Email: {email}")).size(12));
            }
            for phone in &data.phone_numbers {
                card = card.push(text(format!("Phone: {phone}")).size(12));
            }
            let address = persona.identity_address.clone();
            card = card.push(
                button(text("Edit data").size(12))
                    .padding(6)
                    .style(styles::button::base_layer_2_rounded_with_shadow)
                    .on_press(Message::EditPersona(address).into()),
            );
        }

        container(card)
            .padding(10)
            .width(Length::Fill)
            .style(styles::container::weak_layer_2_rounded_with_shadow)
            .into()
    }

    fn edit_form(&'a self) -> Element<'a, AppMessage> {
        let edit = self.editing.as_ref().expect("edit_form called while editing");
        let field = |placeholder: &'a str, value: &'a str, on_input: fn(String) -> Message| {
            text_input(placeholder, value)
                .padding(8)
                .style(styles::text_input::base_layer_1_rounded)
                .on_input(move |input| on_input(input).into())
        };

        let save = button(text("Save").size(12).center())
            .padding(6)
            .style(styles::button::primary)
            .on_press(Message::SaveData.into());
        let cancel = button(text("Cancel").size(12).center())
            .padding(6)
            .style(styles::button::base_layer_2_rounded_with_shadow)
            .on_press(Message::CancelEdit.into());

        column![
            row![
                field("Given name", &edit.given, Message::EditGiven),
                field("Family name", &edit.family, Message::EditFamily),
            ]
            .spacing(8),
            field("Email", &edit.email, Message::EditEmail),
            field("Phone", &edit.phone, Message::EditPhone),
            row![save, cancel].spacing(8),
        ]
        .spacing(8)
        .into()
    }
}
