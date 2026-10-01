//! The flight instruments.
use graticule::instrument::{self, Marker, Side};
use graticule::widget::{self, bar, group, indicator, label, lamp};
use graticule::{Element, Face, px, style};
use iced::widget::{canvas, column, container, row, space};
use iced::{Alignment, Length};

use crate::attitude::Attitude;
use crate::{Message, Telemetry};
use iced::Widget as _;

const COLUMN: f32 = 132.0;

pub fn view<'a>(telemetry: &Telemetry, video: usize) -> Element<'a, Message> {
    let left = column![
        group(
            "PROPELLANT",
            column![
                label("LOX TANK").style(style::text::muted),
                gauges(&telemetry.oxidiser),
                space::vertical().height(px::GAP),
                label("LH2 TANK").style(style::text::muted),
                gauges(&telemetry.fuel),
            ]
            .spacing(px::TIGHT),
        )
        .width(Length::Fill),
        group(
            "MAIN ENGINE",
            column![
                instrument::dial(0.0..=1.0, telemetry.thrust)
                    .ticks(4)
                    .redline(0.85)
                    .height(40.0),
                row![
                    label("THRUST").style(style::text::muted),
                    space::horizontal(),
                    label(format!("{:03.0}%", telemetry.thrust * 100.0)),
                ],
            ]
            .spacing(px::TIGHT),
        )
        .width(Length::Fill),
        group(
            "VIDEO IN",
            widget::selector([(0, "A"), (1, "B"), (2, "C")], Some(video), Message::Video),
        )
        .width(Length::Fill),
    ]
    .spacing(px::WIDE)
    .width(COLUMN);

    let right = column![
        group(
            "THERMAL",
            column![
                widget::reading("CABIN", format!("{:+05.1}°C", telemetry.temperature.0)),
                widget::reading("HULL ", format!("{:+05.0}°C", telemetry.temperature.1)),
            ]
            .spacing(px::TIGHT),
        )
        .width(Length::Fill),
        group(
            "POWER",
            column![
                bus("BUS A", telemetry.bus[0]),
                bus("BUS B", telemetry.bus[1]),
                bus("BUS C", telemetry.bus[2]),
            ]
            .spacing(px::TIGHT),
        )
        .width(Length::Fill),
        group(
            "DOWNLINK",
            column![
                row![
                    bar(0.0..=30.0, telemetry.snr)
                        .segments(3)
                        .redline(26.0)
                        .style(widget::bar::live),
                    label(format!("{:02.0}dB", telemetry.snr)),
                ]
                .spacing(px::GAP)
                .align_y(Alignment::Center),
            ],
        )
        .width(Length::Fill),
        group(
            "ANNUNCIATOR",
            column![
                row![
                    indicator("GUIDE", true),
                    space::horizontal(),
                    indicator("DOCK", false),
                ],
                row![
                    indicator("RCS", telemetry.roll.abs() > 10.0),
                    space::horizontal(),
                    row![
                        lamp::lamp(telemetry.bus[2] < 23.0).style(lamp::alarm),
                        label("VOLT")
                    ]
                    .spacing(px::GAP)
                    .align_y(Alignment::Center),
                ],
            ]
            .spacing(px::TIGHT),
        )
        .width(Length::Fill),
    ]
    .spacing(px::WIDE)
    .width(COLUMN);

    let altitude = instrument::tape(telemetry.altitude, 1.0)
        .face(Face::DISPLAY)
        .format(|value| format!("{value:03.0}"))
        .width(56.0);

    let acceleration = instrument::tape(telemetry.acceleration, 1.0)
        .face(Face::DISPLAY)
        .marker(Marker::Pointer(Side::Right))
        .format(|value| format!("{value:02.0}"))
        .width(48.0);

    let globe = canvas(Attitude::of(telemetry))
        .width(Length::Fill)
        .height(Length::Fill);

    let centre = column![
        row![
            column![caption("ALT KM"), altitude].spacing(px::TIGHT),
            globe,
            column![caption("ACC M/S²"), acceleration].spacing(px::TIGHT),
        ]
        .spacing(px::GAP)
        .height(Length::Fill),
        widget::divider(),
        footer(telemetry),
    ]
    .spacing(px::GAP);

    container(
        row![left, centre, right]
            .spacing(px::FAR)
            .height(Length::Fill),
    )
    .padding(px::WIDE)
    .style(style::container::ground)
    .boxed()
}

fn caption<'a>(text: &'a str) -> Element<'a, Message> {
    label(text).style(style::text::muted).boxed()
}

fn gauges<'a>(levels: &[f32]) -> Element<'a, Message> {
    column(levels.iter().map(|level| {
        row![
            bar(0.0..=1.0, *level),
            label(format!("{:03.0}", level * 100.0)),
        ]
        .spacing(px::GAP)
        .align_y(Alignment::Center)
        .boxed()
    }))
    .spacing(px::TIGHT)
    .boxed()
}

fn bus<'a>(name: &'a str, volts: f32) -> Element<'a, Message> {
    row![
        label(name).style(style::text::muted),
        bar(22.0..=30.0, volts).style(widget::bar::ink),
        label(format!("{volts:04.1}V")),
    ]
    .spacing(px::GAP)
    .align_y(Alignment::Center)
    .boxed()
}

fn footer<'a>(telemetry: &Telemetry) -> Element<'a, Message> {
    let field = |name: &'a str, value: String| widget::field(name, label(value));

    row![
        widget::inverse(label("GRATICULE")),
        field("MODE", "AUTO".to_owned()),
        field("ORBIT", format!("{:05.1} KM", telemetry.altitude)),
        field("INCLINATION", "51.64°".to_owned()),
        field("HEADING", format!("{:03.0}°", telemetry.yaw)),
    ]
    .spacing(px::FAR)
    .boxed()
}
