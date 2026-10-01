//! Every widget of the toolkit, in every state it has.
//!
//! The page shows the toolkit alone: nothing on it is the console's own.
use iced::Widget as _;
use iced::widget::{column, container, progress_bar, row, slider, space, table};
use iced::{Alignment, Color, Length};
use quadrille::widget::{self, bar, button, group, indicator, knob, label, lamp, tab};
use quadrille::{Element, Face, Palette, Theme, icon, px, style};

/// The state of the widgets on the page.
#[derive(Debug, Clone)]
pub struct Kit {
    armed: bool,
    lights: bool,
    level: f32,
    query: String,
    rate: Option<Rate>,
    position: usize,
    mode: Mode,
}

impl Default for Kit {
    fn default() -> Self {
        Self {
            armed: true,
            lights: false,
            level: 0.62,
            query: String::new(),
            rate: Some(Rate::Normal),
            position: 1,
            mode: Mode::Auto,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Armed(bool),
    Lights(bool),
    Level(f32),
    Query(String),
    Rate(Rate),
    Position(usize),
    Mode(Mode),
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rate {
    Slow,
    Normal,
    Fast,
}

impl Rate {
    const ALL: [Self; 3] = [Self::Slow, Self::Normal, Self::Fast];
}

impl std::fmt::Display for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Slow => "SLOW",
            Self::Normal => "NORMAL",
            Self::Fast => "FAST",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Auto,
    Manual,
    Off,
}

impl Kit {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Armed(armed) => self.armed = armed,
            Message::Lights(lights) => self.lights = lights,
            Message::Level(level) => self.level = level,
            Message::Query(query) => self.query = query.to_uppercase(),
            Message::Rate(rate) => self.rate = Some(rate),
            Message::Position(position) => self.position = position,
            Message::Mode(mode) => self.mode = mode,
            Message::Pressed => self.armed = !self.armed,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let buttons = group(
            "BUTTONS",
            column![
                row![
                    button("EXECUTE").on_press(Message::Pressed),
                    button("ENGAGED")
                        .style(style::button::engaged)
                        .on_press(Message::Pressed),
                    button("DISABLED"),
                ]
                .spacing(px::GAP),
                row![
                    tab("F1 LIVE", true).on_press(Message::Pressed),
                    tab("F2 IDLE", false).on_press(Message::Pressed),
                    iced::widget::button(label("GHOST"))
                        .padding(Face::BODY.padding(4, 2, 2))
                        .style(style::button::ghost)
                        .on_press(Message::Pressed),
                ]
                .spacing(px::GAP),
                widget::segmented(
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

        let percent = (self.level * 100.0).round() as i32;
        let turn = |value: i32| Message::Level(value as f32 / 100.0);

        let knobs = group(
            "KNOBS",
            row![
                knob(0..=100, percent, 5, turn),
                knob(0..=100, percent, 5, turn).diameter(19),
                widget::field("LEVEL", label(format!("{percent:03}%"))),
            ]
            .spacing(px::WIDE)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill);

        let icons = group(
            "ICONS",
            column(icon::ALL.chunks(8).map(|chunk| {
                row(chunk
                    .iter()
                    .map(|(_, sprite)| widget::icon(*sprite).boxed()))
                .spacing(px::WIDE)
                .boxed()
            }))
            .spacing(px::GAP),
        )
        .width(Length::Fill);

        let files = group(
            "TABLE",
            widget::table(
                [
                    table::column(label("FILE").style(style::text::muted), |file: File| {
                        label(file.name)
                    }),
                    table::column(label("SIZE").style(style::text::muted), |file: File| {
                        label(file.size)
                    })
                    .align_x(iced::Alignment::End),
                    table::column(label("AGE").style(style::text::muted), |file: File| {
                        label(file.age)
                    })
                    .align_x(iced::Alignment::End),
                ],
                FILES,
            ),
        )
        .width(Length::Fill);

        let inputs = group(
            "INPUT",
            column![
                widget::text_input("SEARCH", &self.query).on_input(Message::Query),
                widget::pick_list(Rate::ALL, self.rate, Message::Rate),
                widget::checkbox("MASTER ARM", self.armed, Message::Armed),
                widget::toggler("FLOODLIGHTS", self.lights).on_toggle(Message::Lights),
                row![
                    widget::radio("AUTO", Mode::Auto, Some(self.mode), Message::Mode),
                    widget::radio("MANUAL", Mode::Manual, Some(self.mode), Message::Mode),
                    widget::radio("OFF", Mode::Off, Some(self.mode), Message::Mode),
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

        let left = column![buttons, lamps, ink]
            .spacing(px::FAR)
            .width(Length::FillPortion(1));
        let middle = column![gauges, knobs, icons, files]
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

/// A row of the table: a file, its size and how long ago it changed.
#[derive(Debug, Clone, Copy)]
struct File {
    name: &'static str,
    size: &'static str,
    age: &'static str,
}

const FILES: [File; 3] = [
    File {
        name: "LOG.TXT",
        size: "12K",
        age: "3M",
    },
    File {
        name: "CORE.BIN",
        size: "2.4M",
        age: "2H",
    },
    File {
        name: "MAP.DAT",
        size: "640K",
        age: "1D",
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
