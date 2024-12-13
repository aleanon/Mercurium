use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, button, column, container, qr_code, row, text},
    Element, Length, Task,
};
use ravault_iced_theme::styles;
use types::{
    address::{AccountAddress, Address},
    debug_info, UnwrapUnreachable,
};

use crate::{app::AppMessage, unlocked::app_view};

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

    pub fn update(&mut self, message: Message) -> Task<AppMessage> {
        match message {
            Message::CopyAddress(address) => {
                self.notification =
                    Notification::Success("Address copied to clipboard".to_string());
                return iced::clipboard::write(address);
            }
        }
    }

    pub fn view(&'a self) -> Element<'a, AppMessage> {
        let close = button(text(Bootstrap::XLg).font(BOOTSTRAP_FONT).size(18))
            .on_press(app_view::Message::CloseOverlay.into())
            .style(button::text);

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
                        text(Bootstrap::XLg).font(BOOTSTRAP_FONT)
                    )
                    .padding(5),
                )
                .center_x(Length::Fill)
                .center_y(50)
                .style(styles::container::notification_success)
                .into(),
                Notification::Error(string) => container(
                    column!(
                        text(string),
                        widget::Space::new(Length::Fill, 1),
                        text(Bootstrap::XLg).font(BOOTSTRAP_FONT)
                    )
                    .padding(5),
                )
                .center_x(Length::Fill)
                .center_y(50)
                .style(styles::container::notification_error)
                .into(),
            }
        };

        let address = text(self.address.truncate_long()).size(14);
        let copy_icon = text(Bootstrap::Copy).font(BOOTSTRAP_FONT).size(14);
        let address_button = button(
            row!(address, copy_icon)
                .spacing(2)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::CopyAddress(self.address.to_string()).into())
        .style(button::text);

        let barcode_address_container = container(
            column!(barcode, address_button)
                .align_x(iced::Alignment::Center)
                .spacing(15),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(10);

        let content = column!(close, notification_box, barcode_address_container);

        container(content)
            .width(400)
            .height(400)
            .padding(1)
            .style(styles::container::overlay_inner)
            .into()
    }
}
