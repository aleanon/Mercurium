use iced::{
    theme::Button,
    widget::{self, text::LineHeight, text_input::Id},
    Element, Length,
};


use types::crypto::Password;


use crate::message::{login_message::LoginMessage, Message};

// #[derive(Debug, Clone)]
// pub enum LoginMessage {
//     TextInputChanged(String),
//     Login,
// }

// impl<'a> LoginMessage {
//     pub fn process(self, app: &'a mut App) -> Command<Message> {
//         let mut command = Command::none();

//         match self {
//             LoginMessage::TextInputChanged(mut string) => {
//                 if let State::Locked(ref mut loginscreen) = app.state {
//                     loginscreen.password.clear();
//                     loginscreen.password.push_str(string.as_str());
//                     string.zeroize()
//                 }
//             }
//             LoginMessage::Login => {
//                 if let State::Locked(ref login) = app.state {
//                     // let salt
//                     let (key, _salt) = login.password.derive_new_db_encryption_key().unwrap();
//                     //take the password, verify and create encryption key and decrypt the database

//                     if let Err(err) = app.login() {
//                         match err {
//                             AppError::Fatal(_) => app.state = State::Error(err),
//                             AppError::NonFatal(_) => { /*In app notification*/ }
//                         }
//                     };

//                     // if let Err(err) = app.action_tx.send(Action::LoadDatabase(key)) {
//                     //     app.state = State::Error(AppError::Fatal(Box::new(err)))
//                     // }

//                     if let Some(ref channel) = app.action_tx {
//                         command = {
//                             let mut connection = channel.clone();
//                             Command::perform(
//                                 async move { connection.send(Action::LoadDatabase(key)).await },
//                                 |_| Message::None,
//                             )
//                         };
//                     }
//                 }
//             }
//         }
//         command
//     }
// }

// impl Into<Message> for LoginMessage {
//     fn into(self) -> Message {
//         Message::Login(self)
//     }
// }

#[derive(Debug, Clone)]
pub struct LoginScreen {
    pub(crate) password: Password,
}

impl<'a> LoginScreen {
    pub fn new() -> Self {
        Self {
            password: Password::new(),
        }
    }

    pub fn password(&self) -> &Password {
        &self.password
    }

    pub fn view(&self) -> Element<'a, Message> {
        let text_field = widget::text_input("Enter Password", &self.password.as_str())
            //.password()
            .secure(true)
            .width(250)
            .line_height(LineHeight::Relative(2.))
            .on_submit(LoginMessage::Login.into())
            .size(15)
            .id(Id::new("password_input"))
            .on_input(|value| LoginMessage::TextInputChanged(value).into());

        let login_button = widget::Button::new(
            widget::text("Login")
                .size(15)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
        .height(30)
        .width(100)
        .style(Button::Primary)
        .on_press(LoginMessage::Login.into());

        let col = widget::column![text_field, login_button]
            .height(Length::Shrink)
            .width(Length::Shrink)
            .align_items(iced::Alignment::Center)
            .spacing(30);

        widget::container(col)
            .center_x()
            .center_y() 
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
