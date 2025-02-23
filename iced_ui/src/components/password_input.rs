use std::{fmt::Debug, mem};

use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{alignment::Vertical, widget::{self, button, column, container, row, text, text_input, Container}, Length};
use ravault_iced_theme::styles;
use types::crypto::Password;
use zeroize::Zeroize;


const ELEMENT_SPACING: u16 = 5;
const ROW_PADDING: u16 = 2;
const RULE_HEIGHT: u16 = 1;
const TEXT_SIZE: u16 = 15;


#[derive(Clone)]
pub enum Message {
    Set(String),
    ToggleView,
    Submit
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Set(_) => write!(f, "Set(*)"),
            Message::ToggleView => write!(f, "ToggleView"),
            Message::Submit => write!(f, "Submit"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasswordInput {
    password: Password,
    view_password: bool,
}

impl<'a> PasswordInput {
    pub fn new() -> Self {
        Self {
            password: Password::new(),
            view_password: false,
        }
    }

    pub fn from_password(password: Password) -> Self {
        Self {
            password,
            view_password: false,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Set(value) => self.input(value),
            Message::ToggleView => self.toggle_view(),
            Message::Submit => {/*Catch in the parent if needed*/}
        }
    }

    pub fn view(&self) -> Container<'a, Message> {
        let input = widget::text_input("Enter Password", self.password.as_str())
            .on_input(Message::Set)
            .on_paste(Message::Set)
            .on_submit(Message::Submit)
            .width(Length::FillPortion(9))
            .size(TEXT_SIZE)
            .secure(!self.view_password)
            .style(styles::text_input::borderless);

        let view_password_icon = if self.view_password {
            Bootstrap::EyeSlash
        } else {
            Bootstrap::Eye
        };

        let toggle_view_password = button(text(view_password_icon).font(BOOTSTRAP_FONT))
            .on_press(Message::ToggleView)
            .width(Length::FillPortion(1))
            .padding(0)
            .style(button::text);

        let input_and_button = row![input, toggle_view_password]
            .align_y(Vertical::Center)
            .padding(ROW_PADDING)
            .spacing(ELEMENT_SPACING);

        let rule = widget::Rule::horizontal(RULE_HEIGHT)
            .style(styles::rule::text_input_rule);

        let content = column![input_and_button, rule];

        container(content)
            .style(styles::container::password_input)
    }

    fn toggle_view(&mut self) {
        self.view_password = !self.view_password;
    }

    fn input(&mut self, mut input: String) {
        self.password.replace(&input);
        input.zeroize();
    }

    pub fn take_password(&mut self) -> Password {
        mem::take(&mut self.password)
    }

    pub fn pw_as_str(&self) -> &str {
        self.password.as_str()
    }
}

pub fn password_input<'a, Func, Message>(
    password: &str,
    view_password: bool,
    on_toggle: Message,
    on_input: Func,
    on_paste: Func,
    on_submit: Message,
) -> container::Container<'a, Message>
where
    Func: 'a + Fn(String) -> Message,
    Message: Clone + 'a,
{
    let input = text_input("Enter Password", password)
        .on_input(on_input)
        .on_paste(on_paste)
        .on_submit(on_submit)
        .width(Length::FillPortion(9))
        .size(TEXT_SIZE)
        .secure(!view_password)
        .style(styles::text_input::borderless);

    let view_password_icon = if view_password {
        Bootstrap::EyeSlash
    } else {
        Bootstrap::Eye
    };

    let toggle_view_password = button(text(view_password_icon).font(BOOTSTRAP_FONT))
        .on_press(on_toggle)
        .width(Length::FillPortion(1))
        .padding(0)
        .style(button::text);

    let input_and_button = row![input, toggle_view_password]
        .align_y(Vertical::Center)
        .padding(ROW_PADDING)
        .spacing(ELEMENT_SPACING);

    let rule = widget::Rule::horizontal(RULE_HEIGHT)
        .style(styles::rule::text_input_rule);

    let content = column![input_and_button, rule];

    container(content)
        .style(styles::container::password_input)
}