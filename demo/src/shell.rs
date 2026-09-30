//! The chassis around every page: a status bar above, soft keys below.
use graticule::widget::{self, divider, inverse, label, soft_key};
use graticule::{Element, Face, Theme, face, px, style};
use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};

use crate::{Message, Page, Telemetry};

/// `page` in the chassis.
pub fn view<'a>(
    page: Page,
    theme: &Theme,
    telemetry: &Telemetry,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    column![
        status(page, telemetry),
        divider(),
        container(content).width(Length::Fill).height(Length::Fill),
        divider(),
        keys(page, theme),
    ]
    .into()
}

fn status<'a>(page: Page, telemetry: &Telemetry) -> Element<'a, Message> {
    let ticks = || label("||||||").style(style::text::faint);

    let title = row![
        ticks(),
        label(face::spaced(page.legend())).style(style::text::accent),
        ticks(),
    ]
    .spacing(f32::from(Face::BODY.advance()) * 2.0);

    let link = telemetry.snr > 12.0;

    let bar = row![
        inverse(label("GRATICULE")),
        label("MK-1 CONSOLE").style(style::text::muted),
        space::horizontal(),
        title,
        space::horizontal(),
        label("MET").style(style::text::muted),
        label(telemetry.clock()),
        widget::indicator("LINK", link),
    ]
    .spacing(px::WIDE)
    .align_y(Alignment::Center);

    container(bar)
        .padding([px::TIGHT, px::WIDE])
        .width(Length::Fill)
        .style(style::container::ground)
        .into()
}

fn keys<'a>(page: Page, theme: &Theme) -> Element<'a, Message> {
    let pages = Page::ALL.into_iter().enumerate().map(|(i, target)| {
        soft_key(format!("F{} {}", i + 1, target.legend()), target == page)
            .on_press(Message::Show(target))
            .into()
    });

    let theme = soft_key(format!("F6 THEME {}", theme.name().to_uppercase()), false)
        .on_press(Message::NextTheme);

    let bar = row(pages)
        .push(space::horizontal())
        .push(theme)
        .spacing(px::GAP)
        .align_y(Alignment::Center);

    container(bar)
        .padding([px::TIGHT, px::WIDE])
        .width(Length::Fill)
        .style(style::container::ground)
        .into()
}
