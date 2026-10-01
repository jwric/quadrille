//! Every widget of the toolkit, in every state it has.
use iced::Widget as _;
use iced::widget::{column, container, progress_bar, row, slider, space, table};
use iced::{Alignment, Color, Length};
use quadrille::instrument::{self, Marker, Side};
use quadrille::widget::{self, bar, group, indicator, key, label, lamp, soft_key};
use quadrille::{Element, Face, Palette, Theme, px, style};

/// The state of the widgets on the page.
#[derive(Debug, Clone)]
pub struct Kit {
    frequency: i64,
    armed: bool,
    lights: bool,
    level: f32,
    callsign: String,
    band: Option<Band>,
    position: usize,
    mode: Mode,
}

impl Default for Kit {
    fn default() -> Self {
        Self {
            frequency: 14_074_000,
            armed: true,
            lights: false,
            level: 0.62,
            callsign: String::new(),
            band: Some(Band::S),
            position: 1,
            mode: Mode::Track,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Frequency(i64),
    Armed(bool),
    Lights(bool),
    Level(f32),
    Callsign(String),
    Band(Band),
    Position(usize),
    Mode(Mode),
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    L,
    S,
    X,
    Ka,
}

impl Band {
    const ALL: [Self; 4] = [Self::L, Self::S, Self::X, Self::Ka];
}

impl std::fmt::Display for Band {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::L => "L BAND",
            Self::S => "S BAND",
            Self::X => "X BAND",
            Self::Ka => "KA BAND",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Track,
    Scan,
    Hold,
}

impl Kit {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Frequency(frequency) => self.frequency = frequency,
            Message::Armed(armed) => self.armed = armed,
            Message::Lights(lights) => self.lights = lights,
            Message::Level(level) => self.level = level,
            Message::Callsign(callsign) => self.callsign = callsign.to_uppercase(),
            Message::Band(band) => self.band = Some(band),
            Message::Position(position) => self.position = position,
            Message::Mode(mode) => self.mode = mode,
            Message::Pressed => self.armed = !self.armed,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let keys = group(
            "KEYS",
            column![
                row![
                    key("EXECUTE").on_press(Message::Pressed),
                    key("ENGAGED")
                        .style(style::button::engaged)
                        .on_press(Message::Pressed),
                    key("DISABLED"),
                ]
                .spacing(px::GAP),
                row![
                    soft_key("F1 LIVE", true).on_press(Message::Pressed),
                    soft_key("F2 IDLE", false).on_press(Message::Pressed),
                    iced::widget::button(label("GHOST"))
                        .padding(Face::BODY.padding(4, 2, 2))
                        .style(style::button::ghost)
                        .on_press(Message::Pressed),
                ]
                .spacing(px::GAP),
                widget::selector(
                    [(0, "OFF"), (1, "LOW"), (2, "MID"), (3, "MAX")],
                    Some(self.position),
                    Message::Position,
                ),
            ]
            .spacing(px::GAP),
        )
        .width(Length::Fill);

        let lamps = group(
            "LAMPS",
            column![
                row![
                    indicator("ARMED", self.armed),
                    indicator("SAFE", !self.armed)
                ]
                .spacing(px::WIDE),
                row![
                    tone("LIVE", lamp::live),
                    tone("WARN", lamp::caution),
                    tone("FAULT", lamp::alarm),
                ]
                .spacing(px::WIDE),
            ]
            .spacing(px::TIGHT),
        )
        .width(Length::Fill);

        let gauges = group(
            "BARS",
            column![
                gauge("PLAIN", bar(0.0..=1.0, self.level).boxed()),
                gauge("SEGMENT", bar(0.0..=1.0, self.level).segments(3).boxed()),
                gauge("REDLINE", bar(0.0..=1.0, self.level).redline(0.7).boxed()),
                gauge(
                    "LIVE",
                    bar(0.0..=1.0, self.level).style(widget::bar::live).boxed()
                ),
                gauge(
                    "INK",
                    bar(0.0..=1.0, self.level).style(widget::bar::ink).boxed()
                ),
                gauge(
                    "LEVEL",
                    slider(0.0..=1.0, self.level, Message::Level)
                        .step(0.01_f32)
                        .boxed()
                ),
                gauge(
                    "PROGRESS",
                    progress_bar(0.0..=1.0, self.level).girth(6.0).boxed()
                ),
            ]
            .spacing(px::TIGHT),
        )
        .width(Length::Fill);

        let inputs = group(
            "INPUT",
            column![
                widget::text_input("CALLSIGN", &self.callsign).on_input(Message::Callsign),
                widget::pick_list(Band::ALL, self.band, Message::Band),
                widget::checkbox("MASTER ARM", self.armed, Message::Armed),
                widget::toggler("FLOODLIGHTS", self.lights).on_toggle(Message::Lights),
                row![
                    widget::radio("TRACK", Mode::Track, Some(self.mode), Message::Mode),
                    widget::radio("SCAN", Mode::Scan, Some(self.mode), Message::Mode),
                    widget::radio("HOLD", Mode::Hold, Some(self.mode), Message::Mode),
                ]
                .spacing(px::WIDE),
            ]
            .spacing(px::GAP),
        )
        .width(Length::Fill);

        let ink = group(
            "INK",
            column![
                row![
                    label("INK"),
                    label("MUTED").style(style::text::muted),
                    label("FAINT").style(style::text::faint),
                    label("LINE").style(style::text::line),
                ]
                .spacing(px::WIDE),
                row![
                    label("ACCENT").style(style::text::accent),
                    label("LIVE").style(style::text::live),
                    label("CAUTION").style(style::text::caution),
                    label("ALARM").style(style::text::alarm),
                ]
                .spacing(px::WIDE),
                row![
                    widget::inverse(label("INVERSE")),
                    container(label("STAMP"))
                        .padding(Face::BODY.padding(2, 0, 0))
                        .style(style::container::stamp),
                    container(label("HIGHLIGHT"))
                        .padding(Face::BODY.padding(2, 0, 0))
                        .style(style::container::highlight),
                ]
                .spacing(px::WIDE),
            ]
            .spacing(px::TIGHT),
        )
        .width(Length::Fill);

        let instruments = group(
            "INSTRUMENTS",
            column![
                row![
                    widget::digits(self.frequency, 9)
                        .group(3, '.')
                        .style(widget::digits::lamp)
                        .on_change(Message::Frequency),
                    label("HZ").style(style::text::muted),
                ]
                .spacing(px::GAP)
                .align_y(Alignment::End),
                row![
                    instrument::dial(0.0..=1.0, self.level)
                        .redline(0.8)
                        .width(64.0)
                        .height(36.0),
                    instrument::tape(self.level * 100.0, 5.0)
                        .format(|value| format!("{value:03.0}"))
                        .width(40.0)
                        .height(64.0),
                    instrument::tape(self.level * 100.0, 5.0)
                        .marker(Marker::Pointer(Side::Right))
                        .format(|value| format!("{value:03.0}"))
                        .width(40.0)
                        .height(64.0),
                ]
                .spacing(px::WIDE),
            ]
            .spacing(px::GAP),
        )
        .width(Length::Fill);

        let passes = group(
            "TABLE",
            widget::table(
                [
                    table::column(label("PASS").style(style::text::muted), |pass: Pass| {
                        label(pass.name)
                    }),
                    table::column(label("AOS").style(style::text::muted), |pass: Pass| {
                        label(pass.rise)
                    }),
                    table::column(label("ELEV").style(style::text::muted), |pass: Pass| {
                        label(format!("{:02}°", pass.elevation))
                    })
                    .align_x(iced::Alignment::End),
                ],
                PASSES,
            ),
        )
        .width(Length::Fill);

        let left = column![keys, lamps, ink]
            .spacing(px::FAR)
            .width(Length::FillPortion(1));
        let middle = column![gauges, instruments, passes]
            .spacing(px::FAR)
            .width(Length::FillPortion(1));
        let right = column![inputs, palette(), spacing()]
            .spacing(px::FAR)
            .width(Length::FillPortion(1));

        container(widget::scroll(row![left, middle, right].spacing(px::FAR)).width(Length::Fill))
            .padding(px::WIDE)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::container::ground)
            .boxed()
    }
}

/// A pass of a satellite over the station.
#[derive(Debug, Clone, Copy)]
struct Pass {
    name: &'static str,
    rise: &'static str,
    elevation: u8,
}

const PASSES: [Pass; 3] = [
    Pass {
        name: "NOAA-19",
        rise: "14:02",
        elevation: 42,
    },
    Pass {
        name: "METEOR-M2",
        rise: "15:37",
        elevation: 7,
    },
    Pass {
        name: "ISS",
        rise: "16:11",
        elevation: 68,
    },
];

fn tone<'a>(name: &'a str, style: fn(&Theme, bool) -> lamp::Style) -> Element<'a, Message> {
    row![lamp::lamp(true).style(style), label(name)]
        .spacing(px::GAP)
        .align_y(Alignment::Center)
        .boxed()
}

fn gauge<'a>(name: &'a str, widget: Element<'a, Message>) -> Element<'a, Message> {
    row![label(name).style(style::text::muted).width(54.0), widget]
        .spacing(px::GAP)
        .align_y(Alignment::Center)
        .boxed()
}

/// A palette role: its name and how to read it.
type Role = (&'static str, fn(&Palette) -> Color);

fn palette<'a>() -> Element<'a, Message> {
    let roles: [Role; 15] = [
        ("VOID", |p| p.void),
        ("GROUND", |p| p.ground),
        ("RAISED", |p| p.raised),
        ("HOVER", |p| p.hover),
        ("EDGE", |p| p.edge),
        ("INK", |p| p.ink),
        ("MUTED", |p| p.muted),
        ("FAINT", |p| p.faint),
        ("LINE", |p| p.line),
        ("ACCENT", |p| p.accent),
        ("ON ACCENT", |p| p.on_accent),
        ("HIGHLIGHT", |p| p.highlight),
        ("LIVE", |p| p.live),
        ("CAUTION", |p| p.caution),
        ("ALARM", |p| p.alarm),
    ];

    let swatches = roles.map(|(name, role)| {
        row![
            container(space())
                .width(10.0)
                .height(8.0)
                .style(move |theme: &Theme| {
                    let palette = theme.palette();

                    container::background(role(palette)).border(style::hairline(palette.edge))
                }),
            label(name).style(style::text::muted),
        ]
        .spacing(px::GAP)
        .align_y(Alignment::Center)
        .boxed()
    });

    let (first, second): (Vec<_>, Vec<_>) = swatches
        .into_iter()
        .enumerate()
        .partition(|(i, _)| i % 2 == 0);

    group(
        "PALETTE",
        row![
            column(first.into_iter().map(|(_, swatch)| swatch)).spacing(px::TIGHT),
            column(second.into_iter().map(|(_, swatch)| swatch)).spacing(px::TIGHT),
        ]
        .spacing(px::WIDE),
    )
    .width(Length::Fill)
    .boxed()
}

fn spacing<'a>() -> Element<'a, Message> {
    let steps = [
        ("HAIR", px::HAIR),
        ("TIGHT", px::TIGHT),
        ("GAP", px::GAP),
        ("WIDE", px::WIDE),
        ("FAR", px::FAR),
    ];

    group(
        "SPACING",
        column(steps.map(|(name, step)| {
            row![
                label(name).style(style::text::muted).width(36.0),
                container(space())
                    .width(step)
                    .height(6.0)
                    .style(|theme: &Theme| container::background(theme.palette().accent)),
                label(format!("{step:.0}PX")).style(style::text::faint),
            ]
            .spacing(px::GAP)
            .align_y(Alignment::Center)
            .boxed()
        }))
        .spacing(px::TIGHT),
    )
    .width(Length::Fill)
    .boxed()
}
