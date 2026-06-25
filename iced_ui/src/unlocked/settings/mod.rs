use deps::*;

use iced::{
    Alignment, Element, Length, Padding,
    widget::{self, button, column, container, row, text, text_input},
};
use types::GatewayConfig;
use zeroize::Zeroize;

use crate::{App, app::AppMessage, styles, unlocked::app_view};

/// Local input messages routed through `AppView::update` to mutate this view's state.
#[derive(Debug, Clone)]
pub enum Message {
    InputBackupPassword(String),
    InputPin(String),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::SettingsViewMessage(self))
    }
}

/// Settings changes routed to `App::apply_settings` (mutates + persists the live profile, or
/// performs an app-level action like exporting an encrypted backup).
#[derive(Debug, Clone)]
pub enum Action {
    SwitchGateway(GatewayConfig),
    SetAppLockEnabled(bool),
    SetPin(String),
    SetDeveloperMode(bool),
    ExportBackup(String),
    ImportBackup(String),
}

#[derive(Debug)]
pub struct SettingsView {
    backup_password: String,
    pin: String,
}

impl<'a> SettingsView {
    pub fn new() -> Self {
        Self {
            backup_password: String::new(),
            pin: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::InputBackupPassword(mut input) => {
                self.backup_password.zeroize();
                self.backup_password = input.clone();
                input.zeroize();
            }
            Message::InputPin(mut input) => {
                self.pin.zeroize();
                self.pin = input.clone();
                input.zeroize();
            }
        }
    }

    pub fn view(&'a self, app: &'a App) -> Element<'a, AppMessage> {
        let profile = &app.profile;
        let header = text("Settings").size(20).width(Length::Fill).center();

        // --- Gateways: one button per saved gateway, current highlighted. ---
        let gateway_buttons = profile.gateways.saved.iter().map(|gateway| {
            let is_current = gateway.url == profile.gateways.current.url;
            let label = format!("{:?} — {}", gateway.network, gateway.url);
            let mut btn = button(text(label).size(13)).width(Length::Fill).padding(8);
            btn = if is_current {
                btn.style(styles::button::primary)
            } else {
                btn.style(styles::button::base_layer_2_rounded_with_shadow)
                    .on_press(AppMessage::Settings(Action::SwitchGateway(gateway.clone())))
            };
            btn.into()
        });
        let gateways = Self::section("Gateway", column(gateway_buttons).spacing(6).into());

        // --- Security toggles. ---
        let app_lock = row![
            text("App lock (PIN)").width(Length::Fill),
            widget::Toggler::new(profile.app_preferences.security.is_app_lock_enabled)
                .on_toggle(|on| AppMessage::Settings(Action::SetAppLockEnabled(on))),
        ]
        .align_y(Alignment::Center);

        let dev_mode = row![
            text("Developer mode").width(Length::Fill),
            widget::Toggler::new(profile.app_preferences.security.is_developer_mode_enabled)
                .on_toggle(|on| AppMessage::Settings(Action::SetDeveloperMode(on))),
        ]
        .align_y(Alignment::Center);

        // --- PIN entry, shown only while app lock is enabled. ---
        let mut security_items = column![app_lock].spacing(10);
        if profile.app_preferences.security.is_app_lock_enabled {
            let pin_set = profile.app_preferences.security.app_lock.is_some();
            let status = if pin_set { "PIN is set" } else { "No PIN set yet" };
            let pin_input = text_input("New PIN", &self.pin)
                .secure(true)
                .padding(10)
                .style(styles::text_input::base_layer_1_rounded)
                .on_input(|input| Message::InputPin(input).into());
            let set_pin = button(text(if pin_set { "Change PIN" } else { "Set PIN" }).center())
                .padding(10)
                .style(styles::button::primary)
                .on_press_maybe(
                    (!self.pin.is_empty())
                        .then(|| AppMessage::Settings(Action::SetPin(self.pin.clone()))),
                );
            security_items = security_items.push(
                column![
                    text(status).size(11).style(styles::text::muted),
                    row![pin_input, set_pin].spacing(10).align_y(Alignment::Center),
                ]
                .spacing(6),
            );
        }
        security_items = security_items.push(dev_mode);

        let security = Self::section("Security", security_items.into());

        // --- Encrypted profile backup (seed-free). ---
        let password_input = text_input("Backup password", &self.backup_password)
            .secure(true)
            .padding(10)
            .style(styles::text_input::base_layer_1_rounded)
            .on_input(|input| Message::InputBackupPassword(input).into());
        let has_password = !self.backup_password.is_empty();
        let export_button = button(text("Export").center().width(Length::Fill))
            .width(Length::Fill)
            .padding(10)
            .style(styles::button::primary)
            .on_press_maybe(has_password.then(|| {
                AppMessage::Settings(Action::ExportBackup(self.backup_password.clone()))
            }));
        let import_button = button(text("Restore").center().width(Length::Fill))
            .width(Length::Fill)
            .padding(10)
            .style(styles::button::base_layer_2_rounded_with_shadow)
            .on_press_maybe(has_password.then(|| {
                AppMessage::Settings(Action::ImportBackup(self.backup_password.clone()))
            }));
        let backup = Self::section(
            "Backup",
            column![
                text("Encrypted with this password; contains no seed phrase.")
                    .size(11)
                    .style(styles::text::muted),
                password_input,
                row![export_button, import_button].spacing(10),
            ]
            .spacing(8)
            .into(),
        );

        let dapps = Self::section(
            "Connected dApps",
            text(format!("{} authorized", profile.authorized_dapps.len()))
                .style(styles::text::muted)
                .into(),
        );

        let content = column![header, gateways, security, backup, dapps]
            .spacing(20)
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

    fn section(title: &'a str, body: Element<'a, AppMessage>) -> Element<'a, AppMessage> {
        container(column![text(title).size(15), body].spacing(10))
            .padding(15)
            .width(Length::Fill)
            .style(styles::container::weak_layer_2_rounded_with_shadow)
            .into()
    }
}
