use iced::{
    theme,
    widget::{button, column, container, qr_code, row, text},
    Element, Length,
};
use ravault_iced_theme::styles;
use types::AccountAddress;

use crate::{
    message::{
        app_view_message::{overlay_message::OverlayMessage, AppViewMessage},
        common_message::CommonMessage,
        Message,
    },
    App,
};

#[derive(Debug)]
pub struct Receive {
    pub address: AccountAddress,
    pub qr_code: qr_code::Data,
}

impl Clone for Receive {
    fn clone(&self) -> Self {
        let address = self.address.clone();
        let qr_code = qr_code::Data::new(&address.as_str()).unwrap();
        Self { address, qr_code }
    }
}

impl<'a> Receive {
    pub fn new(address: AccountAddress) -> Self {
        Self {
            qr_code: qr_code::Data::new(&address.as_str()).unwrap(),
            address,
        }
    }

    pub fn view(&'a self, app: &'a App) -> Element<'a, Message> {
        let close = button(
            text(iced_aw::BootstrapIcon::XLg)
                .font(iced_aw::BOOTSTRAP_FONT)
                .size(18),
        )
        .on_press(AppViewMessage::CloseOverlay.into())
        .style(theme::Button::Text);

        let close = container(close)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right);
        let barcode = qr_code(&self.qr_code);

        let address = text(&self.address.truncate_long()).size(14);
        let copy_icon = text(iced_aw::BootstrapIcon::Copy)
            .font(iced_aw::BOOTSTRAP_FONT)
            .size(14);
        let address_button = button(
            row!(address, copy_icon)
                .spacing(2)
                .align_items(iced::Alignment::Center),
        )
        .on_press(CommonMessage::CopyToClipBoard(self.address.to_string()).into())
        .style(theme::Button::Text);

        let barcode_address_container = container(
            column!(barcode, address_button)
                .align_items(iced::Alignment::Center)
                .spacing(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

        let content = column!(close, barcode_address_container)
            .spacing(10)
            .padding(10);

        container(content)
            .width(400)
            .height(400)
            .style(styles::container::OverlayInner::style)
            .into()
    }
}
