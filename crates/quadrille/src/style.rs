//! Styles for iced's built-in widgets, drawn from the [`Theme`]'s palette.
//!
//! Every style is flat and square: surfaces are separated by shade and by
//! 1 px [`edge`](crate::Palette::edge) hairlines, never by radius or shadow.
//! The `default` function of each module is what a widget gets without a
//! `.style(..)` call; the others are alternatives to pass to it.
use iced_widget::core::{Border, Color};

/// A square border.
pub fn border(color: Color, width: f32) -> Border {
    Border {
        color,
        width,
        radius: 0.0.into(),
    }
}

/// A square 1 px border.
pub fn hairline(color: Color) -> Border {
    border(color, 1.0)
}

/// Styles for text.
pub mod text {
    use iced_widget::text::Style;

    use crate::Theme;

    /// Text in the colour it inherits.
    pub fn default(_theme: &Theme) -> Style {
        Style::default()
    }

    /// Text in the ink colour.
    pub fn ink(theme: &Theme) -> Style {
        colored(theme.palette().ink)
    }

    /// A label: present, not competing.
    pub fn muted(theme: &Theme) -> Style {
        colored(theme.palette().muted)
    }

    /// An engraving: disabled, or the part of a line one skims past.
    pub fn faint(theme: &Theme) -> Style {
        colored(theme.palette().faint)
    }

    /// Schematic annotation, in the line colour.
    pub fn line(theme: &Theme) -> Style {
        colored(theme.palette().line)
    }

    /// State: selected, engaged, powered.
    pub fn accent(theme: &Theme) -> Style {
        colored(theme.palette().accent)
    }

    /// Live data.
    pub fn live(theme: &Theme) -> Style {
        colored(theme.palette().live)
    }

    /// Worth a look.
    pub fn caution(theme: &Theme) -> Style {
        colored(theme.palette().caution)
    }

    /// Wrong.
    pub fn alarm(theme: &Theme) -> Style {
        colored(theme.palette().alarm)
    }

    fn colored(color: iced_widget::core::Color) -> Style {
        Style { color: Some(color) }
    }
}

/// Styles for containers.
pub mod container {
    use iced_widget::container::Style;

    use super::hairline;
    use crate::Theme;

    /// No background, no border.
    pub fn default(_theme: &Theme) -> Style {
        Style::default()
    }

    /// A panel's face, with no border of its own: panels are separated by the
    /// rules between them.
    pub fn ground(theme: &Theme) -> Style {
        Style::default().background(theme.palette().ground)
    }

    /// The void, for the glass of a display.
    pub fn void(theme: &Theme) -> Style {
        Style::default().background(theme.palette().void)
    }

    /// A bezel around glass: the void inside a hairline.
    pub fn inset(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style::default()
            .background(palette.void)
            .border(hairline(palette.edge))
    }

    /// A raised face inside a hairline, like a key that cannot be pressed.
    pub fn plate(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style::default()
            .background(palette.raised)
            .border(hairline(palette.edge))
    }

    /// A hairline and nothing else.
    pub fn outline(theme: &Theme) -> Style {
        Style::default().border(hairline(theme.palette().edge))
    }

    /// An inverse block: the accent behind text in [`on_accent`], which is
    /// how a quadrille interface emphasises instead of with a bold weight.
    ///
    /// [`on_accent`]: crate::Palette::on_accent
    pub fn inverse(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style::default()
            .background(palette.accent)
            .color(palette.on_accent)
    }

    /// The ink behind text in the ground colour: an inverse block with no
    /// signal meaning.
    pub fn stamp(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style::default()
            .background(palette.ink)
            .color(palette.ground)
    }

    /// A highlight behind text.
    pub fn highlight(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style::default()
            .background(palette.highlight)
            .color(palette.on(palette.highlight))
    }

    /// A tooltip: a raised face inside a hairline.
    pub fn tooltip(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style::default()
            .background(palette.raised)
            .border(hairline(palette.edge))
            .color(palette.ink)
    }
}

/// Styles for buttons.
pub mod button {
    use iced_widget::button::{Status, Style};
    use iced_widget::core::{Background, Color};

    use super::hairline;
    use crate::Theme;

    /// A button: a raised face in a hairline that lifts a step under the
    /// cursor and drops to the ground while pressed.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let (face, legend) = match status {
            Status::Active => (palette.raised, palette.ink),
            Status::Hovered => (palette.hover, palette.ink),
            Status::Pressed => (palette.ground, palette.ink),
            Status::Disabled => (palette.ground, palette.faint),
        };

        Style {
            background: Some(face.into()),
            text_color: legend,
            border: hairline(palette.edge),
            ..Style::default()
        }
    }

    /// A button that is on: the accent face with its legend knocked out.
    pub fn engaged(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let face = match status {
            Status::Pressed => palette.raised,
            _ => palette.accent,
        };

        Style {
            background: Some(face.into()),
            text_color: palette.on_accent,
            border: hairline(palette.accent),
            ..Style::default()
        }
    }

    /// No face until the cursor is over it: a row in a list, a link-like
    /// button.
    pub fn ghost(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let (face, legend): (Option<Background>, Color) = match status {
            Status::Active => (None, palette.ink),
            Status::Hovered => (Some(palette.raised.into()), palette.ink),
            Status::Pressed => (Some(palette.hover.into()), palette.ink),
            Status::Disabled => (None, palette.faint),
        };

        Style {
            background: face,
            text_color: legend,
            ..Style::default()
        }
    }

    /// One segment of a segmented control: lit in the accent when it is the
    /// one selected, a plain face otherwise.
    pub fn segment(selected: bool) -> impl Fn(&Theme, Status) -> Style {
        move |theme, status| {
            let palette = theme.palette();

            let (face, legend) = if selected {
                (palette.raised, palette.accent)
            } else {
                match status {
                    Status::Active => (palette.ground, palette.muted),
                    Status::Hovered | Status::Pressed => (palette.hover, palette.ink),
                    Status::Disabled => (palette.ground, palette.faint),
                }
            };

            Style {
                background: Some(face.into()),
                text_color: legend,
                ..Style::default()
            }
        }
    }

    /// A tab: the legend alone until the cursor is over it, the accent while
    /// it is the one in use.
    pub fn tab(active: bool) -> impl Fn(&Theme, Status) -> Style {
        move |theme, status| {
            let palette = theme.palette();

            let (face, legend, edge): (Option<Background>, Color, Color) = if active {
                (
                    Some(palette.accent.into()),
                    palette.on_accent,
                    palette.accent,
                )
            } else {
                match status {
                    Status::Active => (None, palette.muted, palette.edge),
                    Status::Hovered => (Some(palette.raised.into()), palette.ink, palette.edge),
                    Status::Pressed => (Some(palette.hover.into()), palette.ink, palette.edge),
                    Status::Disabled => (None, palette.faint, palette.edge),
                }
            };

            Style {
                background: face,
                text_color: legend,
                border: hairline(edge),
                ..Style::default()
            }
        }
    }
}

/// Styles for text inputs.
pub mod text_input {
    use iced_widget::text_input::{Status, Style};

    use super::hairline;
    use crate::Theme;
    use crate::theme::mix;

    /// A field: the ground in a hairline that brightens under the cursor and
    /// takes the accent while the keyboard is in it.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let (edge, value) = match status {
            Status::Active => (palette.edge, palette.ink),
            Status::Hovered => (palette.muted, palette.ink),
            Status::Focused { .. } => (palette.accent, palette.ink),
            Status::Disabled => (palette.edge, palette.faint),
        };

        Style {
            background: palette.void.into(),
            border: hairline(edge),
            placeholder: palette.faint,
            value,
            selection: mix(palette.void, palette.accent, 0.4),
        }
    }
}

/// Styles for text editors.
pub mod text_editor {
    use iced_widget::text_editor::{Status, Style};

    use super::hairline;
    use crate::Theme;
    use crate::theme::mix;

    /// A field, as for a text input.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let (edge, value) = match status {
            Status::Active => (palette.edge, palette.ink),
            Status::Hovered => (palette.muted, palette.ink),
            Status::Focused { .. } => (palette.accent, palette.ink),
            Status::Disabled => (palette.edge, palette.faint),
        };

        Style {
            background: palette.void.into(),
            border: hairline(edge),
            placeholder: palette.faint,
            value,
            selection: mix(palette.void, palette.accent, 0.4),
        }
    }
}

/// Styles for scrollables.
pub mod scrollable {
    use iced_widget::container;
    use iced_widget::core::Shadow;
    use iced_widget::scrollable::{AutoScroll, Rail, Scroller, Status, Style};

    use super::{border, hairline};
    use crate::Theme;

    /// A bare rail with a scroller in the edge colour, lit while it is held.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let (vertical, horizontal) = match status {
            Status::Active { .. } => (palette.edge, palette.edge),
            Status::Hovered {
                is_vertical_scrollbar_hovered,
                is_horizontal_scrollbar_hovered,
                ..
            } => (
                if is_vertical_scrollbar_hovered {
                    palette.muted
                } else {
                    palette.edge
                },
                if is_horizontal_scrollbar_hovered {
                    palette.muted
                } else {
                    palette.edge
                },
            ),
            Status::Dragged {
                is_vertical_scrollbar_dragged,
                is_horizontal_scrollbar_dragged,
                ..
            } => (
                if is_vertical_scrollbar_dragged {
                    palette.accent
                } else {
                    palette.edge
                },
                if is_horizontal_scrollbar_dragged {
                    palette.accent
                } else {
                    palette.edge
                },
            ),
        };

        let rail = |scroller: iced_widget::core::Color| Rail {
            background: None,
            border: border(palette.edge, 0.0),
            scroller: Scroller {
                background: scroller.into(),
                border: border(scroller, 0.0),
            },
        };

        Style {
            container: container::Style::default(),
            vertical_rail: rail(vertical),
            horizontal_rail: rail(horizontal),
            gap: None,
            auto_scroll: AutoScroll {
                background: palette.raised.into(),
                border: hairline(palette.edge),
                shadow: Shadow::default(),
                icon: palette.ink,
            },
        }
    }
}

/// Styles for rules.
pub mod rule {
    use iced_widget::rule::{FillMode, Style};

    use crate::Theme;

    /// A hairline in the edge colour, snapped to the grid.
    pub fn default(theme: &Theme) -> Style {
        solid(theme.palette().edge)
    }

    /// A hairline in the schematic line colour.
    pub fn line(theme: &Theme) -> Style {
        solid(theme.palette().line)
    }

    /// A hairline in the faint ink.
    pub fn faint(theme: &Theme) -> Style {
        solid(theme.palette().faint)
    }

    fn solid(color: iced_widget::core::Color) -> Style {
        Style {
            color,
            radius: 0.0.into(),
            fill_mode: FillMode::Full,
            snap: true,
        }
    }
}

/// Styles for pick lists.
pub mod pick_list {
    use iced_widget::pick_list::{Status, Style};

    use super::hairline;
    use crate::Theme;

    /// A key's face, lifting a step under the cursor and while open.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let (face, text) = match status {
            Status::Active => (palette.raised, palette.ink),
            Status::Hovered | Status::Opened { .. } => (palette.hover, palette.ink),
            Status::Disabled => (palette.raised, palette.faint),
        };

        Style {
            text_color: text,
            placeholder_color: palette.faint,
            handle_color: palette.muted,
            background: face.into(),
            border: hairline(palette.edge),
        }
    }
}

/// Styles for the menus of pick lists and combo boxes.
pub mod menu {
    use iced_widget::core::Shadow;
    use iced_widget::overlay::menu::Style;

    use super::hairline;
    use crate::Theme;

    /// A raised list in a hairline; the option under the cursor is inverse.
    pub fn default(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style {
            background: palette.raised.into(),
            border: hairline(palette.edge),
            text_color: palette.ink,
            selected_text_color: palette.on_accent,
            selected_background: palette.accent.into(),
            shadow: Shadow::default(),
        }
    }
}

/// Styles for checkboxes.
pub mod checkbox {
    use iced_widget::checkbox::{Status, Style};

    use super::hairline;
    use crate::Theme;

    /// A square: the void when clear, the accent when checked.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let (checked, hovered, disabled) = match status {
            Status::Active { is_checked } => (is_checked, false, false),
            Status::Hovered { is_checked } => (is_checked, true, false),
            Status::Disabled { is_checked } => (is_checked, false, true),
        };

        let face = if disabled {
            palette.ground
        } else if checked {
            palette.accent
        } else {
            palette.void
        };

        Style {
            background: face.into(),
            icon_color: palette.on_accent,
            border: hairline(if hovered { palette.muted } else { palette.edge }),
            text_color: Some(if disabled { palette.faint } else { palette.ink }),
        }
    }
}

/// Styles for radio buttons.
pub mod radio {
    use iced_widget::radio::{Status, Style};

    use crate::Theme;

    /// The void in a hairline, with an accent dot when selected.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let hovered = matches!(status, Status::Hovered { .. });

        Style {
            background: palette.void.into(),
            dot_color: palette.accent,
            border_width: 1.0,
            border_color: if hovered { palette.muted } else { palette.edge },
            text_color: Some(palette.ink),
        }
    }
}

/// Styles for togglers.
pub mod toggler {
    use iced_widget::toggler::{Status, Style};

    use crate::Theme;

    /// A square track with a square thumb that takes the accent when on.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let (on, hovered, disabled) = match status {
            Status::Active { is_toggled } => (is_toggled, false, false),
            Status::Hovered { is_toggled } => (is_toggled, true, false),
            Status::Disabled { is_toggled } => (is_toggled, false, true),
        };

        let thumb = if disabled {
            palette.faint
        } else if on {
            palette.accent
        } else if hovered {
            palette.ink
        } else {
            palette.muted
        };

        Style {
            background: palette.void.into(),
            background_border_width: 1.0,
            background_border_color: palette.edge,
            foreground: thumb.into(),
            foreground_border_width: 0.0,
            foreground_border_color: thumb,
            text_color: Some(if disabled { palette.faint } else { palette.ink }),
            border_radius: Some(0.0.into()),
            padding_ratio: 0.2,
        }
    }
}

/// Styles for sliders.
pub mod slider {
    use iced_widget::slider::{Handle, HandleShape, Rail, Status, Style};

    use super::border;
    use crate::Theme;

    /// A 2 px rail, filled in the accent up to a square handle.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let palette = theme.palette();

        let handle = match status {
            Status::Active => palette.ink,
            Status::Hovered => palette.accent,
            Status::Dragged => palette.accent,
        };

        Style {
            rail: Rail {
                backgrounds: (palette.accent.into(), palette.edge.into()),
                width: 2.0,
                border: border(palette.edge, 0.0),
            },
            handle: Handle {
                shape: HandleShape::Rectangle {
                    width: 4,
                    border_radius: 0.0.into(),
                },
                background: handle.into(),
                border_width: 0.0,
                border_color: handle,
            },
        }
    }
}

/// Styles for progress bars.
pub mod progress_bar {
    use iced_widget::progress_bar::Style;

    use super::hairline;
    use crate::Theme;

    /// The accent filling the void inside a hairline.
    pub fn default(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style {
            background: palette.void.into(),
            bar: palette.accent.into(),
            border: hairline(palette.edge),
        }
    }
}

/// Styles for tables.
pub mod table {
    use iced_widget::table::Style;

    use crate::Theme;

    /// Separators in the edge colour.
    pub fn default(theme: &Theme) -> Style {
        let palette = theme.palette();

        Style {
            separator_x: palette.edge.into(),
            separator_y: palette.edge.into(),
        }
    }
}
