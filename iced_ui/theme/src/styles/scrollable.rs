use iced::{
    border::Radius,
    theme,
    widget::{
        self,
        scrollable::{self, Scrollbar, Scroller},
    },
    Border, Theme,
};

pub struct Scrollable;

impl widget::scrollable::StyleSheet for Scrollable {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> widget::scrollable::Appearance {
        let extended_palette = style.extended_palette();
        let default = style.active(&theme::Scrollable::Default);

        scrollable::Appearance {
            scrollbar: Scrollbar {
                scroller: Scroller {
                    border: Border {
                        radius: Radius::from(10),
                        width: 3.5,
                        color: extended_palette.background.base.color,
                    },
                    color: extended_palette.background.weak.color,
                },
                background: None,
                border: Border::default(),
            },
            ..default
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Appearance {
        let extended_palette = style.extended_palette();
        let active = self.active(style);
        let border_width;
        let color;

        if is_mouse_over_scrollbar {
            border_width = 2.;
            color = extended_palette.background.base.text
        } else {
            border_width = 3.5;
            color = active.scrollbar.scroller.color
        };

        scrollable::Appearance {
            scrollbar: Scrollbar {
                scroller: Scroller {
                    border: Border {
                        width: border_width,
                        ..active.scrollbar.scroller.border
                    },
                    color,
                },
                ..active.scrollbar
            },
            ..active
        }
    }

    fn dragging(&self, style: &Self::Style) -> scrollable::Appearance {
        self.hovered(style, true)
    }
}
