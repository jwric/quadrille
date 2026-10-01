//! The type lab: candidate fonts, compared in use.
//!
//! Every candidate is shown the way the toolkit would use it: a panel of
//! labels, readings and gauges set in it, the same line beside every other
//! candidate, and one glyph under a loupe over its metrics.
mod candidates;

pub use candidates::FONTS;

use iced::widget::canvas::{self, Frame, Geometry};
use iced::widget::{button, canvas as canvas_widget, column, container, row, space};
use iced::{Alignment, Length, Point, Rectangle, Renderer, mouse};
use quadrille::draw::{Anchor, Horizontal, Pattern, Pen, Vertical, rectangle};
use quadrille::widget::{self, bar, group, label, selector};
use quadrille::{Element, Face, Theme, px, style};

use crate::VIEWPORT;
use iced::Widget as _;

/// A font the lab compares, at the size and line where it is crisp.
#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    /// What the lab calls it.
    pub name: &'static str,
    /// Its licence.
    pub licence: &'static str,
    /// Why it is here.
    pub note: &'static str,
    /// The face: the font at its native size, on its densest whole line.
    pub face: Face,
}

/// The toolkit's own faces, then every other candidate.
fn candidates() -> impl Iterator<Item = Candidate> {
    [
        Candidate {
            name: "DEPARTURE TIGHT",
            licence: "OFL-1.1",
            note: "The toolkit's body face: Departure in a 6 px cell.",
            face: Face::BODY,
        },
        Candidate {
            name: "DEPARTURE",
            licence: "OFL-1.1",
            note: "Departure Mono as drawn: a 7 px cell.",
            face: Face::PROSE.with_line(12),
        },
    ]
    .into_iter()
    .chain(candidates::CANDIDATES.iter().copied())
}

/// How many candidates the lab compares.
pub fn count() -> usize {
    candidates().count()
}

fn candidate(index: usize) -> Candidate {
    candidates()
        .nth(index)
        .unwrap_or_else(|| candidates().next().expect("a candidate"))
}

/// What the lab shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// One candidate, in use.
    #[default]
    Specimen,
    /// Every candidate, one line each.
    Ladder,
    /// A small face and a body face working together.
    System,
}

#[derive(Debug, Clone)]
pub struct Lab {
    view: View,
    selected: usize,
    glyph: char,
    small: usize,
}

impl Default for Lab {
    fn default() -> Self {
        Self {
            view: View::Specimen,
            selected: 0,
            glyph: 'Q',
            small: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    View(View),
    Select(usize),
    Glyph(char),
    Small(Choice),
}

/// The characters on the glyph board.
const BOARD: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~°±µΩ←→↑↓▲▼►◄…■□░▒▓█┌┐└┘├┤┬┴┼─│";

impl Lab {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::View(view) => self.view = view,
            Message::Select(index) => self.selected = index,
            Message::Glyph(glyph) => self.glyph = glyph,
            Message::Small(Choice(index)) => self.small = index,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let chosen = candidate(self.selected);

        let list = group(
            "CANDIDATES",
            widget::scroll(
                column(candidates().enumerate().map(|(i, candidate)| {
                    let selected = i == self.selected;
                    let face = candidate.face;

                    button(
                        row![
                            label(candidate.name),
                            space::horizontal(),
                            label(format!("{}×{}", face.advance(), face.line()))
                                .style(style::text::faint),
                        ]
                        .spacing(px::GAP),
                    )
                    .padding([1.0, px::GAP])
                    .width(Length::Fill)
                    .style(style::button::position(selected))
                    .on_press(Message::Select(i))
                    .boxed()
                }))
                .spacing(px::HAIR),
            )
            .height(Length::Fill),
        )
        .width(150.0)
        .height(Length::Fill);

        let views = selector(
            [
                (View::Specimen, "SPECIMEN"),
                (View::Ladder, "LADDER"),
                (View::System, "SYSTEM"),
            ],
            Some(self.view),
            Message::View,
        );

        let body: Element<'_, Message> = match self.view {
            View::Specimen => row![
                specimen(chosen),
                column![
                    group(
                        "LOUPE",
                        canvas_widget(Loupe {
                            glyph: self.glyph,
                            face: chosen.face,
                        })
                        .width(Length::Fill)
                        .height(Length::Fill),
                    )
                    .width(Length::Fill)
                    .height(Length::FillPortion(3)),
                    group(
                        "GLYPHS",
                        widget::scroll(self.board(chosen.face)).width(Length::Fill)
                    )
                    .width(Length::Fill)
                    .height(Length::FillPortion(2)),
                ]
                .spacing(px::WIDE)
                .width(Length::FillPortion(2)),
            ]
            .spacing(px::FAR)
            .boxed(),
            View::Ladder => ladder(self.selected),
            View::System => system(candidate(self.small), chosen, self.small),
        };

        container(
            row![
                list,
                column![views, body].spacing(px::WIDE).width(Length::Fill),
            ]
            .spacing(px::FAR),
        )
        .padding(px::WIDE)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::container::ground)
        .boxed()
    }

    fn board(&self, face: Face) -> Element<'_, Message> {
        let glyphs: Vec<char> = BOARD.chars().collect();

        column(glyphs.chunks(12).map(|chunk| {
            row(chunk.iter().map(|&glyph| {
                let selected = glyph == self.glyph;

                button(face.text(glyph.to_string()))
                    .padding([0.0, 2.0])
                    .style(if selected {
                        style::button::engaged
                    } else {
                        style::button::ghost
                    })
                    .on_press(Message::Glyph(glyph))
                    .boxed()
            }))
            .boxed()
        }))
        .boxed()
    }
}

/// One candidate in use: the parts of an instrument panel set in its face.
fn specimen<'a>(candidate: Candidate) -> Element<'a, Message> {
    let face = candidate.face;
    let text = |content: &str| face.text(content.to_owned());
    let muted = |content: &str| face.text(content.to_owned()).style(style::text::muted);
    let gap = f32::from(face.advance());

    let gauges = column(
        [("LOX", 0.66), ("LH2", 0.88), ("N2", 0.31)]
            .into_iter()
            .map(|(name, level)| {
                row![
                    muted(name).width(gap * 4.0),
                    bar(0.0..=1.0, level).height(face.cap()),
                    text(&format!("{:03.0}", level * 100.0)),
                ]
                .spacing(gap)
                .align_y(Alignment::Center)
                .boxed()
            }),
    )
    .spacing(px::TIGHT);

    let readings = column![
        row![
            muted("FREQ"),
            text("14.074 MHz"),
            muted("MODE"),
            text("USB")
        ]
        .spacing(gap),
        row![
            muted("SNR "),
            text("-12 dB    "),
            muted("PWR "),
            text("100 W")
        ]
        .spacing(gap),
        row![
            muted("TEMP"),
            text("+41.5 °C  "),
            muted("TOL "),
            text("±0.5 %")
        ]
        .spacing(gap),
    ]
    .spacing(px::HAIR);

    let keys = row![
        button(text("EXECUTE")).padding(face.padding(4, 2, 2)),
        button(text("HOLD"))
            .padding(face.padding(4, 2, 2))
            .style(style::button::engaged),
        widget::inverse(text("A U T O")),
    ]
    .spacing(gap)
    .align_y(Alignment::Center);

    let prose = column![
        text("The quick brown fox jumps over"),
        text("the lazy dog. 0123456789"),
        text("Downlink nominal; next pass 14:02."),
    ];

    let table_face = face.with_line_box();
    let table = column(
        [
            "┌──────┬──────┐",
            "│ BAND │ SNR  │",
            "├──────┼──────┤",
            "│ 20 m │ -12  │",
            "│ 40 m │ +03  │",
            "└──────┴──────┘",
        ]
        .into_iter()
        .map(|line| table_face.text(line)),
    );

    let (columns, rows) = density(face);

    let facts = column![
        row![
            muted("CELL"),
            text(&format!("{}×{}", face.advance(), face.line()))
        ]
        .spacing(gap),
        row![
            muted("CAP "),
            text(&format!(
                "{}  X {}  DESC {}",
                face.cap(),
                face.x_height(),
                face.depth()
            ))
        ]
        .spacing(gap),
        row![muted("FITS"), text(&format!("{columns}×{rows} at 640×400"))].spacing(gap),
        row![muted("LICENCE"), text(candidate.licence)].spacing(gap),
        face.text(candidate.note).style(style::text::faint),
    ]
    .spacing(px::HAIR);

    group(
        candidate.name,
        column![gauges, readings, keys, prose, table, facts].spacing(px::WIDE),
    )
    .face(face)
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .boxed()
}

/// How many cells of `face` the showcase's page area holds.
fn density(face: Face) -> (u32, u32) {
    let (width, height) = (
        VIEWPORT.width - 2.0 * px::WIDE,
        VIEWPORT.height - 34.0 - 2.0 * px::WIDE,
    );

    (
        (width / f32::from(face.advance())) as u32,
        (height / f32::from(face.line())) as u32,
    )
}

/// A candidate chosen in a pick list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice(usize);

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let candidate = candidate(self.0);
        let face = candidate.face;

        write!(f, "{} {}×{}", candidate.name, face.advance(), face.line())
    }
}

/// Two faces working as one system: the small one for names, units and
/// annotation, the body one for values, and the body at twice its size for the
/// readout that matters most.
fn system<'a>(small: Candidate, body: Candidate, index: usize) -> Element<'a, Message> {
    let (s, b) = (small.face, body.face);
    let display = b.scaled(2);
    let gap = f32::from(b.advance());

    let name = |content: &str| s.text(content.to_owned()).style(style::text::muted);
    let value = |content: &str| b.text(content.to_owned());

    let field = |label: &str, reading: &str| column![name(label), value(reading)];

    let choices: Vec<Choice> = (0..count()).map(Choice).collect();

    let pick = row![
        label("SMALL FACE").style(style::text::muted),
        widget::pick_list(choices, Some(Choice(index)), Message::Small),
        label("BODY FACE").style(style::text::muted),
        label(body.name),
    ]
    .spacing(px::WIDE)
    .align_y(Alignment::Center);

    let readout = row![
        display.text("14.074"),
        column![name("MHZ"), name("VFO A")].spacing(px::HAIR),
    ]
    .spacing(gap)
    .align_y(Alignment::End);

    let fields = row![
        field("MODE", "USB"),
        field("FILTER", "2.4k"),
        field("SNR", "-12 dB"),
        field("POWER", "100 W"),
        field("SWR", "1.3"),
    ]
    .spacing(gap * 2.0);

    let gauges = column(
        [("S METER", 0.72), ("ALC", 0.18), ("COMP", 0.41)]
            .into_iter()
            .map(|(label, level)| {
                row![
                    name(label).width(f32::from(s.advance()) * 8.0),
                    bar(0.0..=1.0, level).height(b.cap()),
                    value(&format!("{:03.0}", level * 100.0)),
                ]
                .spacing(gap)
                .align_y(Alignment::Center)
                .boxed()
            }),
    )
    .spacing(px::TIGHT);

    let log = column![
        row![name("UTC   "), name("CALL    "), name("BAND"), name("RST")].spacing(gap),
        row![
            value("14:02"),
            value("OH2BH   "),
            value("20 m"),
            value("599")
        ]
        .spacing(gap),
        row![
            value("14:09"),
            value("JA1NUT  "),
            value("20 m"),
            value("579")
        ]
        .spacing(gap),
        s.text("Two contacts this hour; the band is closing to the east.")
            .style(style::text::faint),
    ]
    .spacing(px::HAIR);

    column![
        pick,
        group(
            "SYSTEM",
            column![readout, fields, gauges, log].spacing(px::FAR),
        )
        .face(s)
        .width(Length::Fill)
        .height(Length::Fill),
    ]
    .spacing(px::WIDE)
    .boxed()
}

/// Every candidate, set on the same line.
fn ladder<'a>(selected: usize) -> Element<'a, Message> {
    const SAMPLE: &str = "CH1 FREQ 14.074 MHz  SNR -12 dB  Batt 12.6V ░▒▓ ┌─┬─┐";

    let rows = candidates().enumerate().map(|(i, candidate)| {
        let face = candidate.face;

        column![
            label(format!(
                "{}  {}×{}  CAP {}",
                candidate.name,
                face.advance(),
                face.line(),
                face.cap()
            ))
            .style(if i == selected {
                style::text::accent
            } else {
                style::text::faint
            }),
            face.text(SAMPLE),
        ]
        .boxed()
    });

    widget::scroll(column(rows).spacing(px::GAP).width(Length::Fill))
        .height(Length::Fill)
        .boxed()
}

/// One glyph blown up over its metrics.
struct Loupe {
    glyph: char,
    face: Face,
}

impl<Message> canvas::Program<Message, Theme> for Loupe {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let palette = theme.palette();
        let mut frame = Frame::new(renderer, bounds.size());

        {
            let (width, height) = (bounds.width as i32, bounds.height as i32);
            let fit = (height - 24) / i32::from(self.face.line());
            let big = self.face.scaled(fit.clamp(1, 12) as u16);
            let pixel = i32::from(big.scale());
            let unit = big.scale() / self.face.scale();

            let top = (height - i32::from(big.line())).div_euclid(2);
            let left = (width - i32::from(big.advance())).div_euclid(2);
            let small = Face::BODY;

            let mut pen = Pen::new(&mut frame);

            pen.pattern(
                rectangle(left, top, i32::from(big.advance()), i32::from(big.line())),
                Pattern::Grid(pixel as u8),
                Point::new(left, top),
                palette.faint,
            );

            let bands = [
                ("CAP", big.cap_top(), big.cap() / unit),
                ("X", big.baseline() - big.x_height(), big.x_height() / unit),
                ("BASE", big.baseline(), 0),
                ("DESC", big.baseline() + big.depth(), big.depth() / unit),
            ];

            for (name, offset, value) in bands {
                let y = top + i32::from(offset);

                pen.hline(0, width - 1, y, palette.line);
                pen.text(
                    small,
                    name,
                    Point::new(0, y - 2),
                    Anchor::BASELINE_LEFT,
                    palette.muted,
                );
                pen.text(
                    small,
                    value.to_string(),
                    Point::new(width, y - 2),
                    Anchor::BASELINE_RIGHT,
                    palette.muted,
                );
            }

            pen.text(
                big,
                self.glyph.to_string(),
                Point::new(left, top),
                Anchor::TOP_LEFT,
                palette.ink,
            );

            pen.text(
                small,
                format!("U+{:04X}", u32::from(self.glyph)),
                Point::new(0, 0),
                Anchor::TOP_LEFT,
                palette.accent,
            );

            pen.text(
                small,
                format!("{}×{}", self.face.advance(), self.face.line()),
                Point::new(width, 0),
                Anchor::new(Horizontal::Right, Vertical::Top),
                palette.faint,
            );
        }

        vec![frame.into_geometry()]
    }
}
