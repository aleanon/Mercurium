pub mod accounts_view;
pub mod transaction_view;

use std::collections::HashMap;

use iced::{
    widget::{self, button, container, image::Handle, text, Button, Row},
    Element, Length,
};
use types::ResourceAddress;

use crate::{app::App, message::{app_view_message::AppViewMessage, Message}};

use self::{accounts_view::AccountsView, transaction_view::TransactionView};


#[derive(Debug)]
pub enum ActiveTab {
    Accounts(AccountsView),
    Transfer(TransactionView),
}

#[derive(Debug, Clone)]
pub enum TabId {
    Accounts,
    Transfer,
}

#[derive(Debug)]
pub struct AppView {
    pub notification: Option<String>,
    pub active_tab: ActiveTab,
    //pub menu: Menu,
    //pub center_panel: CenterPanel,
    pub resource_icons: HashMap<ResourceAddress, Handle>,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            notification: None,
            active_tab: ActiveTab::Accounts(AccountsView::new()),
            //menu: Menu::new(),
            //center_panel: CenterPanel::new(),
            resource_icons: HashMap::new(),
        }
    }
}

impl<'a> AppView {

    pub fn view(&'a self, app: &'a App) -> Element<'a, Message> {
        let menu = self.menu();

        let center_panel = match self.active_tab {
            ActiveTab::Accounts(ref accounts_view) => container(accounts_view.view(app)),
            ActiveTab::Transfer(ref transaction_view) => container(transaction_view.view(app)),
        }
        .padding(10);

        let menu_center = widget::row![menu, center_panel]
            .width(Length::Fill)
            .height(Length::Fill);

        let panels: Element<'a, Message>;

        if let Some(notification) = &self.notification {
            let notification_widget = Self::notification_widget(notification);

            panels = widget::column![notification_widget, menu_center].into()
        } else {
            panels = menu_center.into()
        }

        container(panels).into()
    }

    fn menu(&self) -> Element<'a, Message> {
        let toggle_theme_button = Self::menu_button("Toggle theme", Message::ToggleTheme);

        let mut accounts_button =
            Self::menu_button("Accounts", AppViewMessage::SelectTab(TabId::Accounts));

        let mut transfer_button =
            Self::menu_button("Transfer", AppViewMessage::SelectTab(TabId::Transfer));

        match self.active_tab {
            ActiveTab::Accounts(_) => accounts_button = Self::set_active_style(accounts_button),
            ActiveTab::Transfer(_) => transfer_button = Self::set_active_style(transfer_button),
        }

        let top_space = widget::Space::new(Length::Fill, 75);

        let buttons = widget::column![
            top_space,
            toggle_theme_button,
            accounts_button,
            transfer_button
        ]
        .width(Length::Fill)
        .height(Length::Shrink)
        .spacing(5)
        .padding(15);

        let scrollable = widget::scrollable(buttons).height(Length::Shrink);

        container(scrollable).height(Length::Fill).width(200).into()
    }

    fn set_active_style(
        button: iced::widget::Button<'a, Message>,
    ) -> iced::widget::Button<'a, Message> {
        button.style(iced::theme::Button::Secondary)
    }

    fn menu_button(content: &str, message: impl Into<Message>) -> Button<'a, Message> {
        button(
            text(content)
                .size(15)
                .line_height(2.)
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        )
        .height(Length::Shrink)
        .width(Length::Fill)
        .on_press(message.into())
    }

    fn notification_widget(content: &str) -> Row<'a, Message> {
        let text = text(content).size(12).line_height(2.);

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let close = container(widget::Button::new("X"))
            .padding(5)
            .width(Length::Shrink)
            .height(Length::Shrink);

        widget::row![text, space, close]
            .width(Length::Fill)
            .padding(5)
    }
}
