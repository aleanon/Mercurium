use iced::{
    border::Radius,
    widget::{
        self,
        scrollable::{default, Rail, Scroller, Status, Style},
    },
    Border, Theme,
};

pub fn vertical_scrollable(theme: &Theme, status: Status) -> Style {
    match status {
        Status::Active => {
            let palette = theme.extended_palette();
            Style {
                container: widget::container::transparent(theme),
                gap: None,

                vertical_rail: Rail {
                    scroller: Scroller {
                        border: Border {
                            radius: Radius::from(10),
                            width: 3.5,
                            color: palette.background.base.color,
                        },
                        color: palette.background.weak.color,
                    },
                    background: None,
                    border: Border::default(),
                },
                ..default(theme, status)
            }
        }
        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        } => {
            _ = is_horizontal_scrollbar_hovered;
            let palette = theme.extended_palette();
            let border_width;
            let color;

            if is_vertical_scrollbar_hovered {
                border_width = 2.;
                color = palette.background.base.text
            } else {
                border_width = 3.5;
                color = palette.background.weak.color
            };

            Style {
                container: widget::container::transparent(theme),
                vertical_rail: Rail {
                    scroller: Scroller {
                        border: Border {
                            radius: Radius::from(10),
                            width: border_width,
                            color: palette.background.base.color,
                        },
                        color,
                    },
                    background: None,
                    border: Border::default(),
                },
                ..default(theme, status)
            }
        }
        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        } => vertical_scrollable(
            theme,
            Status::Hovered {
                is_horizontal_scrollbar_hovered: is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_hovered: is_vertical_scrollbar_dragged,
            },
        ),
    }
}
