use iced::{
    theme,
    widget::{self, button, column, container, qr_code, row, text},
    Command, Element, Length,
};
use ravault_iced_theme::styles;
use types::{debug_info, unwrap_unreachable::UnwrapUnreachable, AccountAddress};

use crate::{app::AppData, app::AppMessage, unlocked::app_view};

use super::overlay;

#[derive(Debug, Clone)]
pub enum Message {
    CopyAddress(String),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::OverlayMessage(
            overlay::Message::ReceiveMessage(self),
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Notification {
    None,
    Success(String),
    Error(String),
}

#[derive(Debug)]
pub struct Receive {
    pub notification: Notification,
    pub address: AccountAddress,
    pub qr_code: qr_code::Data,
}

impl Clone for Receive {
    fn clone(&self) -> Self {
        let address = self.address.clone();

        // Creating qr code should only fail if the data is to large, the address is never to large so unwrap is called
        let qr_code = qr_code::Data::new(&address.as_str())
            .unwrap_unreachable(debug_info!("Failed to create qr code for address"));

        Self {
            address,
            qr_code,
            notification: self.notification.clone(),
        }
    }
}

impl<'a> Receive {
    pub fn new(address: AccountAddress) -> Self {
        Self {
            notification: Notification::None,
            qr_code: qr_code::Data::new(&address.as_str()).unwrap(),
            address,
        }
    }

    pub fn update(&mut self, message: Message, appdata: &'a mut AppData) -> Command<AppMessage> {
        let mut command = Command::none();
        match message {
            Message::CopyAddress(address) => {
                self.notification =
                    Notification::Success("Address copied to clipboard".to_string());
                command = iced::clipboard::write(address)
            }
        }

        command
    }

    pub fn view(&'a self, appdata: &'a AppData) -> Element<'a, AppMessage> {
        let close = button(
            text(iced_aw::Bootstrap::XLg)
                .font(iced_aw::BOOTSTRAP_FONT)
                .size(18),
        )
        .on_press(app_view::Message::CloseOverlay.into())
        .style(theme::Button::Text);

        let close = container(close)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .padding(10);
        let barcode = qr_code(&self.qr_code);

        let notification_box: Element<'a, AppMessage> = {
            match &self.notification {
                Notification::None => widget::Space::new(Length::Fill, 50).into(),
                Notification::Success(string) => container(
                    column!(
                        text(string),
                        widget::Space::new(Length::Fill, 1),
                        text(iced_aw::Bootstrap::XLg).font(iced_aw::BOOTSTRAP_FONT)
                    )
                    .padding(5),
                )
                .center_x()
                .center_y()
                .width(Length::Fill)
                .height(50)
                .style(styles::container::NotificationSuccess::style)
                .into(),
                Notification::Error(string) => container(
                    column!(
                        text(string),
                        widget::Space::new(Length::Fill, 1),
                        text(iced_aw::Bootstrap::XLg).font(iced_aw::BOOTSTRAP_FONT)
                    )
                    .padding(5),
                )
                .center_x()
                .center_y()
                .width(Length::Fill)
                .height(50)
                .style(styles::container::NotificationError::style)
                .into(),
            }
        };

        let address = text(&self.address.truncate_long()).size(14);
        let copy_icon = text(iced_aw::Bootstrap::Copy)
            .font(iced_aw::BOOTSTRAP_FONT)
            .size(14);
        let address_button = button(
            row!(address, copy_icon)
                .spacing(2)
                .align_items(iced::Alignment::Center),
        )
        .on_press(Message::CopyAddress(self.address.to_string()).into())
        .style(theme::Button::Text);

        let barcode_address_container = container(
            column!(barcode, address_button)
                .align_items(iced::Alignment::Center)
                .spacing(15),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(10);

        let content = column!(close, notification_box, barcode_address_container);

        container(content)
            .width(400)
            .height(400)
            .padding(1)
            .style(styles::container::OverlayInner::style)
            .into()
    }
}
