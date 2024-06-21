pub mod accounts_view;
pub mod overlay;
pub mod transaction_view;

use iced::{
    theme,
    widget::{self, button, image::Handle, row, text, Row, Text},
    Element, Length,
};
use ravault_iced_theme::styles::{
    button::{MenuButton, SelectedMenuButton},
    container::{CenterPanel, MainWindow, MenuContainer},
};
use std::collections::{BTreeSet, HashMap};
use types::{Account, ResourceAddress};

use crate::{
    app::App,
    message::{
        app_view_message::{transaction_message::TransactionMessage, AppViewMessage},
        Message,
    },
};

use self::{accounts_view::AccountsView, overlay::Overlay, transaction_view::TransactionView};

const THEME_ICON: &'static [u8] = include_bytes!("../../../icons/theme.png");
const ACCOUNTS_ICON: &'static [u8] = include_bytes!("../../../icons/bank-account.png");
const TRANSACTION_ICON: &'static [u8] = include_bytes!("../../../icons/transfer.png");
const MENU_LOGO: &'static [u8] = include_bytes!("../../../icons/menu_logo.png");

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
    pub overlay: Option<Overlay>,
    //pub menu: Menu,
    //pub center_panel: CenterPanel,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            notification: None,
            active_tab: ActiveTab::Accounts(AccountsView::new()),
            overlay: None,
            //menu: Menu::new(),
            //center_panel: CenterPanel::new(),
        }
    }
}

impl<'a> AppView {
    pub fn view(&'a self, app: &'a App) -> Element<'a, Message> {
        let menu = self.menu(app);

        let center_panel = match self.active_tab {
            ActiveTab::Accounts(ref accounts_view) => widget::container(accounts_view.view(app)),
            ActiveTab::Transfer(ref transaction_view) => {
                widget::container(transaction_view.view(app))
            }
        }
        .padding(10)
        .style(CenterPanel::style)
        .width(Length::Fill)
        .height(Length::Fill);

        let menu_center_row = widget::row![menu, center_panel]
            .width(Length::Fill)
            .height(Length::Fill);

        let panels: Element<'a, Message>;

        if let Some(notification) = &self.notification {
            let notification_widget = Self::notification_widget(notification);

            panels = widget::column![notification_widget, menu_center_row].into()
        } else {
            panels = menu_center_row.into()
        }

        let appview = widget::container(panels).style(MainWindow::style);

        let overlay = self
            .overlay
            .as_ref()
            .and_then(|overlay| Some(overlay.view(app)));

        iced_aw::modal(appview, overlay)
            .on_esc(AppViewMessage::CloseOverlay.into())
            .backdrop(AppViewMessage::CloseOverlay.into())
            .into()
    }

    fn menu(&self, app: &'a App) -> Element<'a, Message> {
        let logo = widget::image(Handle::from_memory(MENU_LOGO))
            .width(100)
            .height(50);
        let logo_container = widget::container(logo)
            .width(Length::Fill)
            .height(75)
            .center_x()
            .center_y();

        let theme_button_text = format!("{}", app.app_data.settings.theme);
        let theme_icon = text(iced_aw::Bootstrap::Palette).font(iced_aw::BOOTSTRAP_FONT);
        let toggle_theme_button =
            Self::menu_button(theme_icon, &theme_button_text, Message::ToggleTheme.into());

        let accounts_icon = text(iced_aw::Bootstrap::PersonVcard).font(iced_aw::BOOTSTRAP_FONT);
        let mut accounts_button = Self::menu_button(
            accounts_icon,
            "Accounts",
            AppViewMessage::SelectTab(TabId::Accounts).into(),
        );

        let transaction_icon = text(iced_aw::Bootstrap::ArrowBarUp).font(iced_aw::BOOTSTRAP_FONT);
        let message = match &self.active_tab {
            ActiveTab::Transfer(_) => {
                AppViewMessage::TransferMessage(TransactionMessage::OverView).into()
            }
            _ => AppViewMessage::SelectTab(TabId::Transfer).into(),
        };
        let mut transaction_button = Self::menu_button(transaction_icon, "Send", message);

        // let mut transaction_button = {
        //     let icon = text(iced_aw::BootstrapIcon::ArrowBarUp).font(iced_aw::BOOTSTRAP_FONT);

        //     let text = text("Send")
        //         .size(15)
        //         .line_height(2.)
        //         .width(Length::Fill)
        //         .horizontal_alignment(iced::alignment::Horizontal::Left);

        //     let content = row![icon, text]
        //         .spacing(10)
        //         .align_items(iced::Alignment::Center);

        //     button(content)
        //         .height(Length::Shrink)
        //         .width(Length::Fill)
        //         .style(theme::Button::custom(MenuButton))
        //         .on_press(AppViewMessage::SelectTab(TabId::Transfer).into())
        // };

        match self.active_tab {
            ActiveTab::Accounts(_) => {
                accounts_button = accounts_button.style(theme::Button::custom(SelectedMenuButton))
            }
            ActiveTab::Transfer(_) => {
                transaction_button =
                    transaction_button.style(theme::Button::custom(SelectedMenuButton))
            }
        }

        let buttons = widget::column![
            logo_container,
            toggle_theme_button,
            accounts_button,
            transaction_button
        ]
        .width(Length::Fill)
        .height(Length::Shrink)
        .spacing(5)
        .padding(15);

        let scrollable = widget::scrollable(buttons).height(Length::Shrink);

        widget::container(scrollable)
            .height(Length::Fill)
            .width(200)
            .style(MenuContainer::style)
            .into()
    }

    fn menu_button(icon: Text<'a>, name: &str, message: Message) -> widget::Button<'a, Message> {
        let text = text(name)
            .size(15)
            .line_height(2.)
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Left);

        let content = row![icon, text]
            .spacing(10)
            .align_items(iced::Alignment::Center);

        button(content)
            .height(Length::Shrink)
            .width(Length::Fill)
            .style(theme::Button::custom(MenuButton))
            .on_press(message)
    }

    fn notification_widget(content: &str) -> Row<'a, Message> {
        let text = text(content).size(12).line_height(2.);

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let close = widget::container(widget::Button::new("X"))
            .padding(5)
            .width(Length::Shrink)
            .height(Length::Shrink);

        widget::row![text, space, close]
            .width(Length::Fill)
            .padding(5)
    }
}
