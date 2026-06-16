use deps::*;

use iced::{
    Alignment, Element, Length, Padding,
    widget::{self, button, column, container, row, text},
};
use types::GatewayConfig;

use crate::{App, app::AppMessage, styles};

/// A settings change routed to `App::apply_settings` (mutates + persists the live profile).
#[derive(Debug, Clone)]
pub enum Action {
    SwitchGateway(GatewayConfig),
    SetAppLockEnabled(bool),
    SetDeveloperMode(bool),
}

#[derive(Debug)]
pub struct SettingsView;

impl<'a> SettingsView {
    pub fn new() -> Self {
        Self
    }

    pub fn view(&'a self, app: &'a App) -> Element<'a, AppMessage> {
        let profile = &app.profile;
        let header = text("Settings").size(20).width(Length::Fill).center();

        // --- Gateways: one button per saved gateway, current highlighted. ---
        let gateway_buttons = profile.gateways.saved.iter().map(|gateway| {
            let is_current = gateway.url == profile.gateways.current.url;
            let label = format!("{:?} — {}", gateway.network, gateway.url);
            let mut btn = button(text(label).size(13))
                .width(Length::Fill)
                .padding(8);
            btn = if is_current {
                btn.style(styles::button::primary)
            } else {
                btn.style(styles::button::base_layer_2_rounded_with_shadow)
                    .on_press(AppMessage::Settings(Action::SwitchGateway(gateway.clone())))
            };
            btn.into()
        });
        let gateways = Self::section(
            "Gateway",
            column(gateway_buttons).spacing(6).into(),
        );

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

        let security = Self::section("Security", column![app_lock, dev_mode].spacing(10).into());

        // --- Authorized dApps (read-only count for now). ---
        let dapps = Self::section(
            "Connected dApps",
            text(format!("{} authorized", profile.authorized_dapps.len()))
                .style(styles::text::muted)
                .into(),
        );

        let content = column![header, gateways, security, dapps]
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
