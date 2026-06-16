use deps::*;

use font_and_icons::{BOOTSTRAP_FONT, Bootstrap, images::MENU_LOGO};
use iced::{
    Element, Length, Task,
    widget::{self, Row, Text, button, image::Handle, row, text},
};
use std::{collections::HashMap, str::FromStr};
use types::{Account, Decimal, RadixDecimal, address::ResourceAddress};
use wallet::{Unlocked, Wallet};

use crate::{App, app::AppMessage, styles};

use super::{
    accounts::{self, accounts_view::AccountsView},
    overlays::{
        add_account::AddAccount,
        overlay::{self, Overlay, SpawnOverlay},
        receive::Receive,
    },
    personas::{self, PersonasView},
    transaction::{self, create_transaction::CreateTransaction},
};

#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(TabId),
    AccountsViewMessage(super::accounts::accounts_view::Message),
    NewTransaction(Option<Account>),
    TransactionMessage(transaction::create_transaction::Message),
    SpawnOverlay(SpawnOverlay),
    CloseOverlay,
    OverlayMessage(overlay::Message),
    PersonasViewMessage(personas::Message),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(self)
    }
}

#[derive(Debug)]
pub enum ActiveTab {
    Accounts(accounts::AccountsView),
    Transfer(CreateTransaction),
    Personas(PersonasView),
}

#[derive(Debug, Clone)]
pub enum TabId {
    Accounts,
    Transfer,
    Personas,
}

#[derive(Debug)]
pub struct AppView {
    pub notification: Option<String>,
    pub active_tab: ActiveTab,
    pub overlay: Option<Overlay>,
}

impl<'a> AppView {
    pub fn new() -> Self {
        Self {
            notification: None,
            active_tab: ActiveTab::Accounts(AccountsView::new()),
            overlay: None,
        }
    }

    pub fn update(&mut self, message: Message, wallet: &mut Wallet<Unlocked>) -> Task<AppMessage> {
        match message {
            Message::SelectTab(tab_id) => self.select_tab(tab_id),
            Message::NewTransaction(from_account) => self.new_transaction(from_account, wallet),
            Message::AccountsViewMessage(accounts_message) => {
                if let ActiveTab::Accounts(view) = &mut self.active_tab {
                    return view.update(accounts_message, wallet);
                }
            }
            Message::TransactionMessage(transfer_message) => {
                if let ActiveTab::Transfer(view) = &mut self.active_tab {
                    return view.update(transfer_message, wallet);
                }
            }
            Message::SpawnOverlay(overlay_type) => match overlay_type {
                SpawnOverlay::AddAccount => {
                    let (add_account_view, task) = AddAccount::new();
                    self.overlay = Some(Overlay::AddAccount(add_account_view));
                    return task;
                }
                SpawnOverlay::Receive(account_address) => {
                    self.overlay = Some(Overlay::Receive(Receive::new(account_address)))
                }
            },
            Message::CloseOverlay => self.overlay = None,
            Message::OverlayMessage(overlay_message) => {
                if let Some(overlay) = &mut self.overlay {
                    return overlay.update(overlay_message, wallet);
                }
            }
            Message::PersonasViewMessage(personas_message) => {
                if let ActiveTab::Personas(view) = &mut self.active_tab {
                    return view.update(personas_message, wallet);
                }
            }
        }

        Task::none()
    }

    fn select_tab(&mut self, tab_id: TabId) {
        match tab_id {
            TabId::Accounts => self.active_tab = ActiveTab::Accounts(accounts::AccountsView::new()),
            TabId::Transfer => {
                self.active_tab = ActiveTab::Transfer(CreateTransaction::new(None, None))
            }
            TabId::Personas => self.active_tab = ActiveTab::Personas(PersonasView::new()),
        }
    }

    fn new_transaction(&mut self, from_account: Option<Account>, wallet: &'a mut Wallet<Unlocked>) {
        match from_account {
            Some(ref account) => {
                let asset_amounts =
                    wallet
                        .fungibles()
                        .get(&account.address)
                        .and_then(|fungibles| {
                            Some(
                                fungibles
                                    .into_iter()
                                    .filter_map(|fungible| {
                                        Some((
                                            fungible.resource_address.clone(),
                                            RadixDecimal::from_str(&fungible.amount).ok()?.into(),
                                        ))
                                    })
                                    .collect::<HashMap<ResourceAddress, Decimal>>(),
                            )
                        });

                self.active_tab =
                    ActiveTab::Transfer(CreateTransaction::new(from_account, asset_amounts));
            }
            None => {
                self.active_tab = ActiveTab::Transfer(CreateTransaction::new(None, None));
            }
        }
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>, app: &'a App) -> Element<'a, AppMessage> {
        let menu = self.menu(wallet, app);

        let center_panel = match self.active_tab {
            ActiveTab::Accounts(ref accounts_view) => widget::container(accounts_view.view(wallet)),
            ActiveTab::Transfer(ref transaction_view) => {
                widget::container(transaction_view.view(wallet))
            }
            ActiveTab::Personas(ref personas_view) => {
                widget::container(personas_view.view(wallet))
            }
        }
        .padding(10)
        .style(styles::container::center_panel)
        .width(Length::Fill)
        .height(Length::Fill);

        let menu_center_row = widget::row![menu, center_panel]
            .width(Length::Fill)
            .height(Length::Fill);

        let panels: Element<'a, AppMessage>;

        if let Some(notification) = &self.notification {
            let notification_widget = Self::notification_widget(notification);

            panels = widget::column![notification_widget, menu_center_row].into()
        } else {
            panels = menu_center_row.into()
        }

        let appview = widget::container(panels).style(styles::container::main_window);

        // let overlay = self
        //     .overlay
        //     .as_ref()
        //     .and_then(|overlay| Some(overlay.view(wallet)));

        // widgets::Modal::new(appview, overlay)
        //     .on_esc(Message::CloseOverlay.into())
        //     .backdrop(Message::CloseOverlay.into())
        //     .into()
        appview.into()
    }

    fn menu(&self, _wallet: &'a Wallet<Unlocked>, app: &'a App) -> Element<'a, AppMessage> {
        let logo = widget::image(Handle::from_bytes(MENU_LOGO))
            .width(100)
            .height(50);
        let logo_container = widget::container(logo).center_x(Length::Fill).center_y(75);

        // let theme_button_text = appdata.settings.theme;
        let theme_icon = text(Bootstrap::Palette).font(BOOTSTRAP_FONT);
        let toggle_theme_button = Self::menu_button(
            theme_icon,
            app.preferences.theme.as_str(),
            AppMessage::ToggleTheme,
        );

        let accounts_icon = text(Bootstrap::PersonVcard).font(BOOTSTRAP_FONT);
        let mut accounts_button = Self::menu_button(
            accounts_icon,
            "Accounts",
            Message::SelectTab(TabId::Accounts).into(),
        );

        let transaction_icon = text(Bootstrap::ArrowBarUp).font(BOOTSTRAP_FONT);
        let message = match &self.active_tab {
            ActiveTab::Transfer(_) => {
                Message::TransactionMessage(transaction::create_transaction::Message::OverView)
                    .into()
            }
            _ => Message::SelectTab(TabId::Transfer).into(),
        };
        let mut transaction_button = Self::menu_button(transaction_icon, "Send", message);

        let personas_icon = text(Bootstrap::PersonBadge).font(BOOTSTRAP_FONT);
        let mut personas_button = Self::menu_button(
            personas_icon,
            "Personas",
            Message::SelectTab(TabId::Personas).into(),
        );

        match self.active_tab {
            ActiveTab::Accounts(_) => {
                accounts_button = accounts_button.style(styles::button::selected_menu_button)
            }
            ActiveTab::Transfer(_) => {
                transaction_button = transaction_button.style(styles::button::selected_menu_button)
            }
            ActiveTab::Personas(_) => {
                personas_button = personas_button.style(styles::button::selected_menu_button)
            }
        }

        let buttons = widget::column![
            logo_container,
            toggle_theme_button,
            accounts_button,
            transaction_button,
            personas_button
        ]
        .width(Length::Fill)
        .height(Length::Shrink)
        .spacing(5)
        .padding(15);

        let scrollable = widget::scrollable(buttons).height(Length::Shrink);

        widget::container(scrollable)
            .height(Length::Fill)
            .width(200)
            .style(styles::container::menu_container)
            .into()
    }

    fn menu_button(
        icon: Text<'a>,
        name: &'a str,
        message: AppMessage,
    ) -> widget::Button<'a, AppMessage> {
        let text = text(name)
            .size(15)
            .line_height(2.)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left);

        let content = row![icon, text]
            .spacing(10)
            .align_y(iced::Alignment::Center);

        button(content)
            .height(Length::Shrink)
            .width(Length::Fill)
            .style(styles::button::menu_button)
            .on_press(message)
    }

    fn notification_widget(content: &'a str) -> Row<'a, AppMessage> {
        let text = text(content).size(12).line_height(2.);

        let space = widget::space::horizontal();

        let close = widget::container(widget::Button::new("X"))
            .padding(5)
            .width(Length::Shrink)
            .height(Length::Shrink);

        widget::row![text, space, close]
            .width(Length::Fill)
            .padding(5)
    }
}
