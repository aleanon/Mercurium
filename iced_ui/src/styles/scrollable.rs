use deps::*;

use iced::{
    border::Radius,
    widget::{
        self,
        scrollable::{default, Rail, Scroller, Status, Style},
    },
    Border,
};

use crate::styles::colors::dark;
use crate::Theme;

pub fn vertical_scrollable_secondary(theme: &Theme, status: Status) -> Style {
    match status {
        Status::Active {
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => {
            let palette = theme.extended_palette();
            Style {
                container: widget::container::transparent(theme),
                gap: None,

                vertical_rail: Rail {
                    scroller: Scroller {
                        border: Border {
                            radius: Radius::from(10),
                            width: 3.5,
                            color: dark::BACKGROUND_SECONDARY,
                        },
                        color: dark::BACKGROUND_PRIMARY,
                    },
                    background: None,
                    border: Border::default(),
                },
                ..default(theme, status)
            }
        }
        Status::Hovered {
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        } => {
            _ = is_horizontal_scrollbar_hovered;
            let palette = theme.extended_palette();
            let border_width;
            let color;

            if is_vertical_scrollbar_hovered {
                border_width = 1.;
                color = dark::STATE_HOVER
            } else {
                border_width = 3.5;
                color = dark::BACKGROUND_PRIMARY
            };

            Style {
                container: widget::container::transparent(theme),
                vertical_rail: Rail {
                    scroller: Scroller {
                        border: Border {
                            radius: Radius::from(10),
                            width: border_width,
                            color: dark::BACKGROUND_SECONDARY,
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
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        } => vertical_scrollable_secondary(
            theme,
            Status::Hovered {
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
                is_horizontal_scrollbar_hovered: is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_hovered: is_vertical_scrollbar_dragged,
            },
        ),
    }
}

pub fn vertical_scrollable_primary(theme: &Theme, status: Status) -> Style {
    match status {
        Status::Active {
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => {
            let palette = theme.extended_palette();
            Style {
                container: widget::container::transparent(theme),
                gap: None,

                vertical_rail: Rail {
                    scroller: Scroller {
                        border: Border {
                            radius: Radius::from(10),
                            width: 3.5,
                            color: dark::BACKGROUND_PRIMARY,
                        },
                        color: dark::BACKGROUND_SECONDARY,
                    },
                    background: None,
                    border: Border::default(),
                },
                ..default(theme, status)
            }
        }
        Status::Hovered {
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        } => {
            _ = is_horizontal_scrollbar_hovered;
            let palette = theme.extended_palette();
            let border_width;
            let color;

            if is_vertical_scrollbar_hovered {
                border_width = 2.;
                color = dark::STATE_HOVER
            } else {
                border_width = 3.5;
                color = dark::BACKGROUND_SECONDARY
            };

            Style {
                container: widget::container::transparent(theme),
                vertical_rail: Rail {
                    scroller: Scroller {
                        border: Border {
                            radius: Radius::from(10),
                            width: border_width,
                            color: dark::BACKGROUND_PRIMARY,
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
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        } => vertical_scrollable_primary(
            theme,
            Status::Hovered {
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
                is_horizontal_scrollbar_hovered: is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_hovered: is_vertical_scrollbar_dragged,
            },
        ),
    }
}
