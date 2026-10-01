//! Technical drawing: a cutaway with callouts, and a block diagram, on a
//! sheet with zones and a title block.
use std::cell::Cell;

use iced::widget::canvas::{Action, Cache, Event, Frame, Geometry};
use iced::widget::{canvas, column, container, row, space, stack};
use iced::{Alignment, Color, Length, Point, Rectangle, Renderer, Size, mouse};
use quadrille::draw::{
    Anchor, Axis, Chain, Dash, Direction, Field, Horizontal, Lettering, Note, Pattern, Pen,
    Polygon, Sheet, Table, Vertical, Wire, fan, junctions, rectangle, shape,
};
use quadrille::theme::mix;
use quadrille::widget::{field, inverse, label};
use quadrille::{Element, Face, Palette, Theme, px, style};

use crate::Telemetry;
use iced::Widget as _;

/// The state of the page.
#[derive(Debug, Clone, Default)]
pub struct Drawing {
    pinned: Option<Pin>,
}

/// A part pinned to the spec plate, and the zone of the sheet it is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pin {
    part: usize,
    zone: (char, u8),
}

/// A message of the page.
#[derive(Debug, Clone)]
pub enum Message {
    /// A part was clicked, or the drawing away from any part.
    Pin(Option<Pin>),
}

impl Drawing {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Pin(pin) => self.pinned = pin,
        }
    }

    pub fn view(&self, telemetry: &Telemetry) -> Element<'_, Message> {
        let sheet = canvas(Border).width(Length::Fill).height(Length::Fill);

        let cutaway = canvas(Cutaway {
            pinned: self.pinned.map(|pin| pin.part),
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let diagram = canvas(Diagram {
            elapsed: telemetry.elapsed,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let right = column![
            diagram,
            space::vertical().height(1.0),
            plate(self.pinned),
            space::vertical().height((Table::new(TITLE_BLOCK).height() - 1) as f32),
        ]
        .width(RIGHT as f32);

        let panes = container(row![cutaway, right].spacing(1)).padding(inset() as f32);

        stack![sheet, panes].boxed()
    }
}

/// Pixels between the page's edge and the sheet's trim line.
const MARGIN: i32 = 3;

/// The width of the column right of the cutaway.
const RIGHT: i32 = 214;

/// The height of the spec plate.
const PLATE: i32 = 48;

/// Roughly the pixels across a zone of the sheet.
const ZONE: i32 = 76;

/// The sheet on a page of `size`.
fn sheet(size: Size<i32>) -> Sheet {
    let bounds = rectangle(
        MARGIN,
        MARGIN,
        size.width - 2 * MARGIN,
        size.height - 2 * MARGIN,
    );
    let zones = |length: i32| (length / ZONE / 2 * 2).clamp(2, 24) as u8;

    Sheet::new(bounds, zones(bounds.width), zones(bounds.height))
}

/// Pixels from the page's edge to the drawing area.
fn inset() -> i32 {
    MARGIN + sheet(Size::new(0, 0)).band()
}

/// The title block, in the sheet's bottom right corner.
const TITLE_BLOCK: &[&[Field<'static>]] = &[
    &[Field::new("TITLE", "BEACON ASSY, S-BAND RELAY")],
    &[
        Field::new("DWG NO", "GR-0413-00").span(3),
        Field::new("SHEET", "3 OF 7").span(2),
        Field::new("SCALE", "AS SHOWN").span(3),
        Field::new("REV", "C").span(1),
    ],
    &[
        Field::new("DRAWN", "JWR 2026-09-14"),
        Field::new("CHECKED", "EK 2026-09-28"),
    ],
];

/// The spec plate: the pinned part, or the assembly when none is.
fn plate<'a>(pinned: Option<Pin>) -> Element<'a, Message> {
    let (title, hint, fields) = match pinned {
        Some(pin) => {
            let part = &PARTS[pin.part];

            (
                part.name,
                "PINNED",
                [
                    ("ITEM", format!("{:02}", pin.part + 1)),
                    ("PART NO", part.number.to_owned()),
                    ("MASS", part.mass.to_owned()),
                    ("ZONE", format!("{}{}", pin.zone.0, pin.zone.1)),
                ],
            )
        }
        None => (
            "BEACON ASSY",
            "CLICK A PART",
            [
                ("ITEMS", PARTS.len().to_string()),
                ("PART NO", "GR-0413-00".to_owned()),
                ("MASS", "6.60 KG".to_owned()),
                ("SHEET", "3 OF 7".to_owned()),
            ],
        ),
    };

    let heading = row![
        inverse(label(title)),
        space::horizontal(),
        label(hint).style(style::text::faint),
    ]
    .align_y(Alignment::Center);

    let fields = row(fields
        .into_iter()
        .map(|(name, value)| field(name, label(value)).boxed()))
    .spacing(px::WIDE);

    container(column![heading, fields].spacing(px::GAP))
        .padding([px::GAP, px::WIDE])
        .width(Length::Fill)
        .height(PLATE as f32)
        .boxed()
}

/// A cache that is cleared when its key changes.
struct Layer<K> {
    cache: Cache,
    key: Cell<Option<K>>,
}

impl<K: Copy + PartialEq> Layer<K> {
    /// The cache, emptied first if it was filled under another key.
    fn keyed(&self, key: K) -> &Cache {
        if self.key.get() != Some(key) {
            self.cache.clear();
            self.key.set(Some(key));
        }

        &self.cache
    }
}

impl<K> Default for Layer<K> {
    fn default() -> Self {
        Self {
            cache: Cache::default(),
            key: Cell::new(None),
        }
    }
}

/// `size` in whole pixels.
fn whole(size: Size) -> Size<i32> {
    Size::new(size.width as i32, size.height as i32)
}

/// The sheet's border, the rules between the panes, and the title block.
struct Border;

impl canvas::Program<Message, Theme> for Border {
    type State = Layer<Palette>;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let palette = *theme.palette();

        let geometry = state.keyed(palette).draw(renderer, bounds.size(), |frame| {
            let sheet = sheet(whole(bounds.size()));
            let inner = sheet.inner();
            let (right, bottom) = (inner.x + inner.width, inner.y + inner.height);
            let divider = right - RIGHT - 1;
            let table = Table::new(TITLE_BLOCK);
            let top = bottom - table.height() + 1;

            let mut pen = Pen::new(frame);

            pen.sheet(&sheet, palette.line, palette.muted);
            pen.vline(divider, inner.y, bottom - 1, palette.line);
            pen.hline(divider + 1, right - 1, top - PLATE - 1, palette.line);
            pen.table(
                &table,
                Point::new(divider, top),
                right - divider + 1,
                palette.line,
                palette.muted,
                palette.ink,
            );

            projection(&mut pen, Point::new(right - 26, top + 11), palette.line);
        });

        vec![geometry]
    }
}

/// The first-angle projection symbol, centred on `at`: a cone side on, and
/// end on to its right.
fn projection(pen: &mut Pen<'_, Renderer>, at: Point<i32>, color: Color) {
    let cone = Polygon::new([
        Point::new(at.x - 14, at.y - 5),
        Point::new(at.x - 4, at.y - 3),
        Point::new(at.x - 4, at.y + 3),
        Point::new(at.x - 14, at.y + 5),
    ]);
    let end = Point::new(at.x + 8, at.y);

    pen.polygon(&cone, color);
    pen.circle(end, 5, color);
    pen.circle(end, 3, color);
    pen.chain(
        Point::new(at.x - 17, at.y),
        Point::new(at.x + 15, at.y),
        Chain::new(9, 1, 1),
        color,
    );
}

/// A shape of a part, in model units.
#[derive(Debug, Clone, Copy)]
enum Shape {
    /// The visible edges of a closed outline.
    Outline(&'static [(i32, i32)]),
    /// The hidden edges of a closed outline.
    Hidden(&'static [(i32, i32)]),
    /// A cut face: contours hatched even-odd, and their edges.
    Section(&'static [&'static [(i32, i32)]]),
    /// A visible circle: its centre and radius.
    Circle((i32, i32), i32),
    /// A hidden circle.
    HiddenCircle((i32, i32), i32),
    /// A visible line through points.
    Line(&'static [(i32, i32)]),
    /// A hidden line through points.
    HiddenLine(&'static [(i32, i32)]),
}

/// A part of the beacon.
struct Part {
    /// The callout, a line per `\n`.
    callout: &'static str,
    /// The name on the spec plate.
    name: &'static str,
    number: &'static str,
    mass: &'static str,
    /// Where its leader starts, in model units.
    target: (i32, i32),
    shapes: &'static [Shape],
}

/// The model's half-width, in model units.
const HALF: i32 = 34;
/// The model's first row.
const TOP: i32 = 4;
/// The model's last row.
const BOTTOM: i32 = 126;

/// The row that leaders above rise from and leaders below fall from.
const PIVOT: i32 = 58;

/// The beacon, drawn back to front. One model unit is 5 mm.
const PARTS: &[Part] = &[
    Part {
        callout: "WHIP ANTENNA",
        name: "WHIP ANTENNA",
        number: "GR-0413-01",
        mass: "0.06 KG",
        target: (-1, 17),
        shapes: &[
            Shape::Outline(&[(-1, 8), (1, 8), (1, 26), (-1, 26)]),
            Shape::Circle((0, 6), 2),
        ],
    },
    Part {
        callout: "RADOME",
        name: "RADOME",
        number: "GR-0413-02",
        mass: "0.21 KG",
        target: (10, 36),
        shapes: &[Shape::Outline(&[(-4, 26), (4, 26), (16, 46), (-16, 46)])],
    },
    Part {
        callout: "FEED HORN",
        name: "FEED HORN",
        number: "GR-0413-03",
        mass: "0.05 KG",
        target: (-6, 39),
        shapes: &[Shape::Hidden(&[(-3, 30), (3, 30), (8, 44), (-8, 44)])],
    },
    Part {
        callout: "SHELL\nAL 6061-T6",
        name: "SHELL",
        number: "GR-0413-04",
        mass: "0.94 KG",
        target: (21, 103),
        shapes: &[Shape::Section(&[&[
            (-22, 46),
            (22, 46),
            (22, 106),
            (20, 106),
            (20, 48),
            (-20, 48),
            (-20, 106),
            (-22, 106),
        ]])],
    },
    Part {
        callout: "STAR TRACKER",
        name: "STAR TRACKER",
        number: "GR-0413-05",
        mass: "0.36 KG",
        target: (-34, 51),
        shapes: &[
            Shape::Outline(&[(-30, 47), (-22, 47), (-22, 55), (-30, 55)]),
            Shape::Outline(&[(-30, 48), (-34, 45), (-34, 57), (-30, 54)]),
            Shape::Circle((-26, 51), 2),
        ],
    },
    Part {
        callout: "TRANSPONDER\nS-BAND",
        name: "TRANSPONDER",
        number: "GR-0413-06",
        mass: "0.42 KG",
        target: (-17, 58),
        shapes: &[
            Shape::Outline(&[(-17, 52), (3, 52), (3, 64), (-17, 64)]),
            Shape::Outline(&[(-14, 50), (-10, 50), (-10, 52), (-14, 52)]),
            Shape::Outline(&[(-6, 50), (-2, 50), (-2, 52), (-6, 52)]),
            Shape::HiddenLine(&[(-15, 58), (1, 58)]),
        ],
    },
    Part {
        callout: "REFERENCE OSC\nOCXO 10 MHZ",
        name: "REFERENCE OSC",
        number: "GR-0413-07",
        mass: "0.12 KG",
        target: (17, 57),
        shapes: &[
            Shape::Outline(&[(7, 52), (17, 52), (17, 62), (7, 62)]),
            Shape::Circle((12, 57), 3),
        ],
    },
    Part {
        callout: "BATTERY\n28 V 12 AH",
        name: "BATTERY",
        number: "GR-0413-08",
        mass: "1.60 KG",
        target: (-15, 76),
        shapes: &[
            Shape::Outline(&[(-15, 68), (15, 68), (15, 82), (-15, 82)]),
            Shape::Line(&[(-5, 68), (-5, 82)]),
            Shape::Line(&[(5, 68), (5, 82)]),
            Shape::Outline(&[(-11, 66), (-9, 66), (-9, 68), (-11, 68)]),
            Shape::Outline(&[(9, 66), (11, 66), (11, 68), (9, 68)]),
        ],
    },
    Part {
        callout: "RADIATOR",
        name: "RADIATOR",
        number: "GR-0413-09",
        mass: "0.58 KG",
        target: (34, 78),
        shapes: &[
            Shape::Outline(&[(22, 56), (34, 62), (34, 94), (22, 100)]),
            Shape::Outline(&[(-22, 56), (-34, 62), (-34, 94), (-22, 100)]),
            Shape::HiddenLine(&[(28, 61), (28, 95)]),
            Shape::HiddenLine(&[(-28, 61), (-28, 95)]),
        ],
    },
    Part {
        callout: "N₂H₄ TANK",
        name: "N₂H₄ TANK",
        number: "GR-0413-10",
        mass: "0.88 KG",
        target: (8, 93),
        shapes: &[Shape::Circle((0, 93), 8), Shape::HiddenCircle((0, 93), 6)],
    },
    Part {
        callout: "HARNESS",
        name: "HARNESS",
        number: "GR-0413-11",
        mass: "0.14 KG",
        target: (-18, 90),
        shapes: &[Shape::HiddenLine(&[
            (-17, 62),
            (-18, 62),
            (-18, 103),
            (-3, 103),
            (-3, 110),
        ])],
    },
    Part {
        callout: "SEPARATION\nRING",
        name: "SEPARATION RING",
        number: "GR-0413-12",
        mass: "0.31 KG",
        target: (24, 108),
        shapes: &[
            Shape::Section(&[
                &[(-24, 106), (-19, 106), (-19, 110), (-24, 110)],
                &[(19, 106), (24, 106), (24, 110), (19, 110)],
            ]),
            Shape::Line(&[(-19, 106), (19, 106)]),
            Shape::Line(&[(-19, 110), (19, 110)]),
        ],
    },
    Part {
        callout: "HEAT SHIELD",
        name: "HEAT SHIELD",
        number: "GR-0413-13",
        mass: "0.74 KG",
        target: (-28, 113),
        shapes: &[Shape::Section(&[
            &[(-30, 110), (-4, 110), (-4, 116), (-27, 116)],
            &[(4, 110), (30, 110), (27, 116), (4, 116)],
        ])],
    },
    Part {
        callout: "THRUSTER 1 N",
        name: "THRUSTER",
        number: "GR-0413-14",
        mass: "0.19 KG",
        target: (6, 122),
        shapes: &[Shape::Outline(&[
            (-3, 110),
            (3, 110),
            (3, 116),
            (8, 126),
            (-8, 126),
            (-3, 116),
        ])],
    },
];

/// The cutaway fitted to a pane: the scale, where the model's origin lands,
/// and the callouts laid out around it.
struct Fit {
    size: Size<i32>,
    scale: i32,
    origin: Point<i32>,
    /// Each part's callout, unless it does not fit on the pane.
    notes: Vec<Option<Note<'static>>>,
}

impl Fit {
    /// Pixels from the model's edge to the callout columns.
    const REACH: i32 = 10;
    /// Pixels a callout column takes.
    const LABELS: i32 = 88;
    /// Pixels under the model for the dimensions and the caption.
    const CAPTION: i32 = 56;

    fn new(size: Size<i32>) -> Self {
        let across = (size.width - 2 * (Self::REACH + Self::LABELS)) / (2 * HALF + 1);
        let down = (size.height - Self::CAPTION - 8) / (BOTTOM - TOP);
        let scale = across.min(down).max(1);

        let height = (BOTTOM - TOP) * scale + Self::CAPTION;
        let origin = Point::new(
            size.width.div_euclid(2),
            (size.height - height).div_euclid(2) - TOP * scale,
        );

        let mut fit = Self {
            size,
            scale,
            origin,
            notes: vec![None; PARTS.len()],
        };

        let pivot = fit.at((0, PIVOT)).y;
        let column = HALF * scale + Self::REACH;

        for left in [true, false] {
            let side: Vec<usize> = (0..PARTS.len())
                .filter(|&i| (PARTS[i].target.0 < 0) == left)
                .collect();
            let mut notes: Vec<_> = side
                .iter()
                .map(|&i| {
                    let label = Lettering::new(Face::BODY, PARTS[i].callout, Color::TRANSPARENT);

                    Note::new(fit.at(PARTS[i].target), label)
                })
                .collect();
            let x = if left {
                origin.x - column
            } else {
                origin.x + column
            };

            fan(&mut notes, x, 6..=size.height - 28, pivot);

            for (&i, note) in side.iter().zip(notes) {
                let bounds = note.bounds();
                let fits = bounds.x >= 0
                    && bounds.y >= 0
                    && bounds.x + bounds.width <= size.width
                    && bounds.y + bounds.height <= size.height;

                fit.notes[i] = fits.then_some(note);
            }
        }

        fit
    }

    /// The pixel of the model point `(x, y)`.
    fn at(&self, (x, y): (i32, i32)) -> Point<i32> {
        Point::new(
            self.origin.x + x * self.scale,
            self.origin.y + y * self.scale,
        )
    }

    fn polygon(&self, points: &[(i32, i32)]) -> Polygon {
        Polygon::new(points.iter().map(|&point| self.at(point)))
    }

    fn section(&self, contours: &[&[(i32, i32)]]) -> Polygon {
        contours
            .iter()
            .fold(Polygon::default(), |polygon, contour| {
                polygon.with(contour.iter().map(|&point| self.at(point)))
            })
    }

    /// Whether `shape` covers `point`, or lies within a pixel or two of it.
    fn covers(&self, shape: &Shape, point: Point<i32>) -> bool {
        match *shape {
            Shape::Outline(points) | Shape::Hidden(points) => self.polygon(points).contains(point),
            Shape::Section(contours) => self.section(contours).contains(point),
            Shape::Circle(centre, radius) | Shape::HiddenCircle(centre, radius) => {
                let centre = self.at(centre);
                let (dx, dy) = (point.x - centre.x, point.y - centre.y);

                dx * dx + dy * dy <= (radius * self.scale).pow(2)
            }
            Shape::Line(points) | Shape::HiddenLine(points) => points.windows(2).any(|pair| {
                let (a, b) = (self.at(pair[0]), self.at(pair[1]));

                near(a, b, point, 2)
            }),
        }
    }

    /// The rectangle holding every shape of `part`.
    fn bounds(&self, part: &Part) -> Rectangle<i32> {
        let mut points = Vec::new();

        for shape in part.shapes {
            match *shape {
                Shape::Outline(list)
                | Shape::Hidden(list)
                | Shape::Line(list)
                | Shape::HiddenLine(list) => points.extend(list.iter().copied()),
                Shape::Section(contours) => {
                    points.extend(contours.iter().copied().flatten().copied());
                }
                Shape::Circle((x, y), r) | Shape::HiddenCircle((x, y), r) => {
                    points.extend([(x - r, y - r), (x + r, y + r)]);
                }
            }
        }

        let pixels: Vec<_> = points.into_iter().map(|point| self.at(point)).collect();
        let left = pixels.iter().map(|p| p.x).min().unwrap_or(0);
        let right = pixels.iter().map(|p| p.x).max().unwrap_or(0);
        let top = pixels.iter().map(|p| p.y).min().unwrap_or(0);
        let bottom = pixels.iter().map(|p| p.y).max().unwrap_or(0);

        rectangle(left, top, right - left + 1, bottom - top + 1)
    }

    /// The part under `point`: the smallest one covering it, or the one
    /// whose callout it is on.
    fn hit(&self, point: Point<i32>) -> Option<usize> {
        let on_note = self.notes.iter().position(|note| {
            note.is_some_and(|note| {
                let bounds = note.bounds();

                (bounds.x - 1..=bounds.x + bounds.width).contains(&point.x)
                    && (bounds.y - 1..=bounds.y + bounds.height).contains(&point.y)
            })
        });

        on_note.or_else(|| {
            PARTS
                .iter()
                .enumerate()
                .filter(|(_, part)| part.shapes.iter().any(|shape| self.covers(shape, point)))
                .min_by_key(|(_, part)| {
                    let bounds = self.bounds(part);

                    bounds.width * bounds.height
                })
                .map(|(i, _)| i)
        })
    }

    /// The zone of the sheet `part`'s target is in, for a pane of this size.
    fn zone(&self, part: usize) -> (char, u8) {
        let inset = inset();
        let page = Size::new(
            self.size.width + 1 + RIGHT + 2 * inset,
            self.size.height + 2 * inset,
        );
        let target = self.at(PARTS[part].target);

        sheet(page)
            .zone(Point::new(target.x + inset, target.y + inset))
            .unwrap_or(('A', 1))
    }
}

/// Whether `point` is within `tolerance` pixels of the segment from `a` to
/// `b`.
fn near(a: Point<i32>, b: Point<i32>, point: Point<i32>, tolerance: i32) -> bool {
    let (dx, dy) = ((b.x - a.x) as f32, (b.y - a.y) as f32);
    let (px, py) = ((point.x - a.x) as f32, (point.y - a.y) as f32);
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        ((px * dx + py * dy) / length).clamp(0.0, 1.0)
    };
    let (ex, ey) = (px - t * dx, py - t * dy);

    ex * ex + ey * ey <= (tolerance * tolerance) as f32
}

/// The cutaway, its callouts, dimensions and caption.
struct Cutaway {
    pinned: Option<usize>,
}

/// The cutaway's state: the part under the pointer, and the drawing as it
/// was last drawn.
#[derive(Default)]
struct Hover {
    part: Option<usize>,
    layer: Layer<(Option<usize>, Option<usize>, Palette)>,
}

impl canvas::Program<Message, Theme> for Cutaway {
    type State = Hover;

    fn update(
        &self,
        state: &mut Hover,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let Event::Mouse(event) = event else {
            return None;
        };

        let under = |fit: &Fit| {
            let position = cursor.position_in(bounds)?;

            fit.hit(Point::new(position.x as i32, position.y as i32))
        };

        match event {
            mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft => {
                let part = under(&Fit::new(whole(bounds.size())));

                (part != state.part).then(|| {
                    state.part = part;
                    Action::request_redraw()
                })
            }
            mouse::Event::ButtonPressed(mouse::Button::Left) if cursor.is_over(bounds) => {
                let fit = Fit::new(whole(bounds.size()));
                let pin = under(&fit)
                    .filter(|&part| Some(part) != self.pinned)
                    .map(|part| Pin {
                        part,
                        zone: fit.zone(part),
                    });

                Some(Action::publish(Message::Pin(pin)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Hover,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let palette = *theme.palette();
        let key = (state.part, self.pinned, palette);

        let geometry = state
            .layer
            .keyed(key)
            .draw(renderer, bounds.size(), |frame| {
                let fit = Fit::new(whole(bounds.size()));
                let lit = |i: usize| state.part == Some(i) || self.pinned == Some(i);
                let mut pen = Pen::new(frame);

                self.annotate(&mut pen, &fit, &palette);

                pen.chain(
                    fit.at((0, TOP - 3)),
                    fit.at((0, BOTTOM + 3)),
                    Chain::CENTRE,
                    palette.line,
                );

                for pass in [false, true] {
                    for (i, part) in PARTS.iter().enumerate().filter(|(i, _)| lit(*i) == pass) {
                        let color = if pass { palette.accent } else { palette.ink };
                        let annotation = if pass { palette.accent } else { palette.line };

                        if self.pinned == Some(i) {
                            for shape in part.shapes {
                                shade(&mut pen, &fit, shape, palette.accent);
                            }
                        }

                        for shape in part.shapes {
                            draw_shape(&mut pen, &fit, shape, color, annotation);
                        }

                        if let Some(note) = fit.notes[i] {
                            let label = Lettering {
                                color: if pass { palette.accent } else { palette.muted },
                                ..note.label
                            };

                            pen.note(&Note { label, ..note }, annotation);
                        }
                    }
                }

                if let Some(pinned) = self.pinned {
                    let bounds = fit.bounds(&PARTS[pinned]);

                    pen.brackets(
                        rectangle(
                            bounds.x - 3,
                            bounds.y - 3,
                            bounds.width + 6,
                            bounds.height + 6,
                        ),
                        4,
                        1,
                        palette.accent,
                    );
                }
            });

        vec![geometry]
    }

    fn mouse_interaction(
        &self,
        state: &Hover,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.part.is_some() && cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl Cutaway {
    /// The dimensions, the caption and the marks around the cutaway.
    fn annotate(&self, pen: &mut Pen<'_, Renderer>, fit: &Fit, palette: &Palette) {
        let face = Face::BODY;
        let bottom = fit.at((0, BOTTOM)).y;
        let centre = fit.origin.x;

        // The nozzle's exit, and the span across the radiators, each with
        // extension lines stopping short of the part.
        let dimensions = [((8, BOTTOM), 6, "80"), ((HALF, 94), 18, "340")];

        for ((x, from), below, text) in dimensions {
            let (left, right) = (fit.at((-x, from)), fit.at((x, from)));
            let y = bottom + below;

            for end in [left, right] {
                pen.vline(end.x, end.y + 2, y + 2, palette.line);
            }

            pen.dimension(
                left.x..=right.x,
                y,
                Lettering::new(face, text, palette.muted),
                palette.line,
            );
        }

        // The mast, from its tip to the radome.
        let (tip, foot) = (fit.at((1, 8)), fit.at((4, 26)));
        let x = fit.at((8, 0)).x;

        pen.hline(tip.x + 2, x + 2, tip.y, palette.line);
        pen.hline(foot.x + 2, x + 2, foot.y, palette.line);
        pen.vertical_dimension(
            x,
            tip.y..=foot.y,
            Lettering::new(face, "90", palette.muted),
            palette.line,
        );

        let caption = bottom + 28;

        pen.text(
            Face::DISPLAY,
            "FIG. 3",
            Point::new(centre, caption),
            Anchor::new(Horizontal::Centre, Vertical::CapTop),
            palette.ink,
        );
        pen.text(
            face,
            format!("SECTION A-A  SCALE {}:1", fit.scale),
            Point::new(centre, caption + 20),
            Anchor::new(Horizontal::Centre, Vertical::CapTop),
            palette.muted,
        );

        let corner = fit.size.height - 12;

        pen.strike(
            face,
            "CONFIDENTIAL",
            Point::new(6, corner),
            Anchor::new(Horizontal::Left, Vertical::CapTop),
            palette.line,
        );
        for (line, text) in ["DIMENSIONS IN MM", "DO NOT SCALE"].into_iter().enumerate() {
            pen.text(
                face,
                text,
                Point::new(fit.size.width - 6, corner - 12 + 12 * line as i32),
                Anchor::new(Horizontal::Right, Vertical::CapTop),
                palette.faint,
            );
        }
    }
}

/// Fills the inside of `shape` with a sparse dither in `color`, as a pinned
/// part is filled.
fn shade(pen: &mut Pen<'_, Renderer>, fit: &Fit, shape: &Shape, color: Color) {
    const DITHER: Pattern = Pattern::Grid(2);

    match *shape {
        Shape::Outline(points) | Shape::Hidden(points) => {
            pen.fill_polygon(&fit.polygon(points), DITHER, fit.origin, color);
        }
        Shape::Circle(centre, radius) | Shape::HiddenCircle(centre, radius) => {
            for (y, from, to) in shape::disc(fit.at(centre), radius * fit.scale) {
                pen.pattern(
                    rectangle(from, y, to - from + 1, 1),
                    DITHER,
                    fit.origin,
                    color,
                );
            }
        }
        Shape::Section(_) | Shape::Line(_) | Shape::HiddenLine(_) => {}
    }
}

/// Draws `shape` with edges in `color` and hatching in `hatch`.
fn draw_shape(pen: &mut Pen<'_, Renderer>, fit: &Fit, shape: &Shape, color: Color, hatch: Color) {
    let scale = fit.scale;

    match *shape {
        Shape::Outline(points) => pen.polygon(&fit.polygon(points), color),
        Shape::Hidden(points) => pen.dashed_polygon(&fit.polygon(points), Dash::HIDDEN, color),
        Shape::Section(contours) => {
            let section = fit.section(contours);

            pen.fill_polygon(&section, Pattern::Hatch(4), fit.origin, hatch);
            pen.polygon(&section, color);
        }
        Shape::Circle(centre, radius) => pen.circle(fit.at(centre), radius * scale, color),
        Shape::HiddenCircle(centre, radius) => {
            pen.dashed_circle(fit.at(centre), radius * scale, Dash::HIDDEN, color);
        }
        Shape::Line(points) => {
            let points: Vec<_> = points.iter().map(|&point| fit.at(point)).collect();

            pen.polyline(&points, color);
        }
        Shape::HiddenLine(points) => {
            for pair in points.windows(2) {
                pen.dashed(fit.at(pair[0]), fit.at(pair[1]), Dash::HIDDEN, color);
            }
        }
    }
}

/// A block of the diagram: its label and its cell in the grid.
struct Block {
    label: &'static str,
    cell: (i32, i32),
}

// Indices into `BLOCKS`.
const DIPLEXER: usize = 0;
const LNA: usize = 1;
const MIXER: usize = 2;
const DEMOD: usize = 3;
const OSCILLATOR: usize = 4;
const AMPLIFIER: usize = 5;
const MODULATOR: usize = 6;
const PROCESSOR: usize = 7;
const POWER: usize = 8;
const COMPUTER: usize = 9;

const BLOCKS: [Block; 10] = [
    Block {
        label: "DIPLX",
        cell: (0, 0),
    },
    Block {
        label: "LNA",
        cell: (1, 0),
    },
    Block {
        label: "MIX",
        cell: (2, 0),
    },
    Block {
        label: "DEMOD",
        cell: (3, 0),
    },
    Block {
        label: "OSC",
        cell: (1, 1),
    },
    Block {
        label: "PA",
        cell: (1, 2),
    },
    Block {
        label: "MOD",
        cell: (2, 2),
    },
    Block {
        label: "DSP",
        cell: (3, 2),
    },
    Block {
        label: "PCU",
        cell: (0, 3),
    },
    Block {
        label: "OBC",
        cell: (3, 3),
    },
];

/// A stop on the signal's way round the transponder.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Stop {
    /// A pixel of a wire.
    Wire(Point<i32>),
    /// Inside a block.
    Block(usize),
    /// Leaving the antenna.
    Radiating,
}

/// The block diagram laid out in a pane.
struct Plan {
    /// The centre of the first block.
    origin: Point<i32>,
    /// Wires, and whether each has an arrowhead at its end.
    wires: Vec<(Wire, bool)>,
    /// Where the wires join.
    joins: Vec<Point<i32>>,
    /// The first pixel of the power bus, which carries a label.
    power: Point<i32>,
    /// The ends of the data bus, drawn heavy with a width mark.
    bus: (Point<i32>, Point<i32>),
    /// Where the antenna's feed ends.
    antenna: Point<i32>,
    /// The signal's way round, one stop per step.
    route: Vec<Stop>,
}

impl Plan {
    /// Pixels from one column of blocks to the next.
    const PITCH: i32 = 54;
    /// Each row's offset from the first.
    const ROWS: [i32; 4] = [0, 40, 80, 128];
    /// The size of a block.
    const BLOCK: (i32, i32) = (37, 17);
    /// The diagram and its caption around the first block's centre.
    const EXTENT: Rectangle<i32> = rectangle(-18, -26, 3 * 54 + 37, 26 + 128 + 8 + 30);
    /// Steps the signal spends in each block.
    const DWELL: usize = 6;

    fn new(size: Size<i32>) -> Self {
        let origin = Point::new(
            (size.width - Self::EXTENT.width).div_euclid(2) - Self::EXTENT.x,
            (size.height - Self::EXTENT.height).div_euclid(2) - Self::EXTENT.y,
        );

        let centre = |block: usize| {
            let (column, row) = BLOCKS[block].cell;

            Point::new(
                origin.x + column * Self::PITCH,
                origin.y + Self::ROWS[row as usize],
            )
        };

        let (half_x, half_y) = (Self::BLOCK.0 / 2 + 1, Self::BLOCK.1 / 2 + 1);
        let left = |block| Point::new(centre(block).x - half_x, centre(block).y);
        let right = |block| Point::new(centre(block).x + half_x, centre(block).y);
        let top = |block| Point::new(centre(block).x, centre(block).y - half_y);
        let bottom = |block| Point::new(centre(block).x, centre(block).y + half_y);

        let across = |from, to| Wire::route(from, to, Axis::Horizontal, Axis::Horizontal);
        let down = |from, to| Wire::route(from, to, Axis::Vertical, Axis::Vertical);

        let feed = top(DIPLEXER);
        let antenna = Point::new(feed.x, feed.y - 9);
        let tee = Point::new(centre(MIXER).x, centre(OSCILLATOR).y);
        let power = across(right(POWER), left(COMPUTER));

        let receive = [
            across(right(DIPLEXER), left(LNA)),
            across(right(LNA), left(MIXER)),
            across(right(MIXER), left(DEMOD)),
            down(bottom(DEMOD), top(PROCESSOR)),
        ];
        let transmit = [
            across(left(PROCESSOR), right(MODULATOR)),
            across(left(MODULATOR), right(AMPLIFIER)),
            Wire::route(
                left(AMPLIFIER),
                bottom(DIPLEXER),
                Axis::Horizontal,
                Axis::Vertical,
            ),
        ];

        let mut wires: Vec<(Wire, bool)> = receive
            .iter()
            .chain(&transmit)
            .map(|wire| (wire.clone(), true))
            .collect();

        wires.extend([
            (power.clone(), true),
            (Wire::start(feed).vertical(antenna.y), false),
            (across(right(OSCILLATOR), tee), false),
            (down(tee, bottom(MIXER)), true),
            (down(tee, top(MODULATOR)), true),
            (
                down(
                    Point::new(centre(AMPLIFIER).x, power.first().y),
                    bottom(AMPLIFIER),
                ),
                true,
            ),
            (
                down(
                    Point::new(centre(MODULATOR).x, power.first().y),
                    bottom(MODULATOR),
                ),
                true,
            ),
        ]);

        let joins = junctions(
            &wires
                .iter()
                .map(|(wire, _)| wire.clone())
                .collect::<Vec<_>>(),
        );

        // Down the feed from the antenna, through the receiver and the
        // processor, and back out through the transmitter.
        let mut route: Vec<Stop> = Vec::new();
        let through = |route: &mut Vec<Stop>, wire: &Wire, block: usize| {
            route.extend(wire.pixels().into_iter().map(Stop::Wire));
            route.extend([Stop::Block(block); Self::DWELL]);
        };

        let mut inbound = Wire::start(feed).vertical(antenna.y).pixels();
        inbound.reverse();
        route.extend(inbound.into_iter().map(Stop::Wire));
        route.extend([Stop::Block(DIPLEXER); Self::DWELL]);

        for (wire, block) in receive.iter().zip([LNA, MIXER, DEMOD, PROCESSOR]) {
            through(&mut route, wire, block);
        }

        for (wire, block) in transmit.iter().zip([MODULATOR, AMPLIFIER, DIPLEXER]) {
            through(&mut route, wire, block);
        }

        route.extend(
            Wire::start(feed)
                .vertical(antenna.y)
                .pixels()
                .into_iter()
                .map(Stop::Wire),
        );
        route.extend([Stop::Radiating; 3 * Self::DWELL]);

        Self {
            origin,
            wires,
            joins,
            power: power.first(),
            bus: (bottom(PROCESSOR), top(COMPUTER)),
            antenna,
            route,
        }
    }

    fn block(&self, block: usize) -> Rectangle<i32> {
        let (column, row) = BLOCKS[block].cell;
        let centre = Point::new(
            self.origin.x + column * Self::PITCH,
            self.origin.y + Self::ROWS[row as usize],
        );

        rectangle(
            centre.x - Self::BLOCK.0 / 2,
            centre.y - Self::BLOCK.1 / 2,
            Self::BLOCK.0,
            Self::BLOCK.1,
        )
    }

    /// Draws a block, its outline and label in `color`.
    fn draw_block(&self, pen: &mut Pen<'_, Renderer>, block: usize, color: Color) {
        let bounds = self.block(block);
        let centre = Point::new(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);

        pen.outline(bounds, color);
        pen.text(
            Face::BODY,
            BLOCKS[block].label,
            centre,
            Anchor::CENTRE,
            color,
        );
    }

    /// Draws the antenna in `color`, and the waves leaving it when
    /// `radiating`.
    fn draw_antenna(&self, pen: &mut Pen<'_, Renderer>, color: Color, radiating: Option<usize>) {
        let tip = self.antenna;
        let dish = Polygon::new([
            Point::new(tip.x - 5, tip.y - 6),
            Point::new(tip.x + 5, tip.y - 6),
            tip,
        ]);

        pen.polygon(&dish, color);

        if let Some(step) = radiating {
            let centre = Point::new(tip.x, tip.y - 6);

            for (i, radius) in [5, 8, 11].into_iter().enumerate() {
                if step >= i * 4 {
                    pen.arc(centre, radius, 320.0, 40.0, color);
                }
            }
        }
    }
}

/// The transponder's block diagram, with a pulse running round it.
struct Diagram {
    elapsed: f32,
}

impl Diagram {
    /// Steps along the route per second.
    const SPEED: f32 = 45.0;
}

impl canvas::Program<Message, Theme> for Diagram {
    type State = Layer<Palette>;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let palette = *theme.palette();
        let plan = Plan::new(whole(bounds.size()));
        let face = Face::BODY;

        let still = state.keyed(palette).draw(renderer, bounds.size(), |frame| {
            let mut pen = Pen::new(frame);

            for (wire, arrow) in &plan.wires {
                pen.wire(wire, palette.line);

                if let (true, Some(heading)) = (*arrow, wire.heading()) {
                    pen.arrowhead(wire.last(), heading, 2, palette.line);
                }
            }

            for &join in &plan.joins {
                pen.junction(join, palette.line);
            }

            // The data bus: three pixels wide, arrowed both ways, with a
            // slash and its width.
            let (from, to) = plan.bus;
            let middle = from.y + (to.y - from.y) / 2;

            pen.fill(
                rectangle(from.x - 1, from.y + 3, 3, to.y - from.y - 5),
                palette.line,
            );
            pen.arrowhead(from, Direction::Up, 3, palette.line);
            pen.arrowhead(to, Direction::Down, 3, palette.line);
            pen.line(
                Point::new(from.x - 5, middle + 5),
                Point::new(from.x + 5, middle - 5),
                palette.line,
            );
            pen.text(
                face,
                "16",
                Point::new(from.x + 6, middle - 3),
                Anchor::BASELINE_LEFT,
                palette.muted,
            );
            pen.text(
                face,
                "TM/TC",
                Point::new(from.x - 8, middle + 1),
                Anchor::RIGHT,
                palette.muted,
            );

            pen.text(
                face,
                "28 V",
                Point::new(plan.power.x + 5, plan.power.y - 2),
                Anchor::BASELINE_LEFT,
                palette.muted,
            );

            for block in 0..BLOCKS.len() {
                plan.draw_block(&mut pen, block, palette.ink);
            }

            plan.draw_antenna(&mut pen, palette.ink, None);
            pen.text(
                face,
                "ANT",
                Point::new(plan.antenna.x + 8, plan.antenna.y - 3),
                Anchor::LEFT,
                palette.muted,
            );

            let caption = plan.block(COMPUTER);
            let caption = Point::new(
                plan.origin.x + Plan::EXTENT.x + (Plan::EXTENT.width - 1) / 2,
                caption.y + caption.height + 12,
            );

            pen.text(
                face,
                "FIG. 4  SIGNAL FLOW",
                caption,
                Anchor::new(Horizontal::Centre, Vertical::CapTop),
                palette.muted,
            );
        });

        let mut frame = Frame::new(renderer, bounds.size());

        {
            let mut pen = Pen::new(&mut frame);
            let step = (self.elapsed.max(0.0) * Self::SPEED) as usize % plan.route.len();
            let trail = mix(palette.line, palette.accent, 0.5);

            match plan.route[step] {
                Stop::Wire(head) => {
                    let tail = plan.route[step.saturating_sub(4)..step]
                        .iter()
                        .filter_map(|stop| match stop {
                            Stop::Wire(pixel) => Some(*pixel),
                            _ => None,
                        });

                    for pixel in tail {
                        pen.pixel(pixel, trail);
                    }

                    pen.junction(head, palette.accent);
                }
                Stop::Block(block) => plan.draw_block(&mut pen, block, palette.accent),
                Stop::Radiating => {
                    let first = plan
                        .route
                        .iter()
                        .position(|stop| *stop == Stop::Radiating)
                        .unwrap_or(step);

                    plan.draw_antenna(&mut pen, palette.accent, Some(step - first));
                }
            }
        }

        vec![still, frame.into_geometry()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether the segments `a` and `b` cross or touch.
    fn crosses(a: (Point<i32>, Point<i32>), b: (Point<i32>, Point<i32>)) -> bool {
        let side = |p: Point<i32>, q: Point<i32>, r: Point<i32>| {
            i64::from(q.x - p.x) * i64::from(r.y - p.y)
                - i64::from(q.y - p.y) * i64::from(r.x - p.x)
        };

        let d1 = side(b.0, b.1, a.0).signum();
        let d2 = side(b.0, b.1, a.1).signum();
        let d3 = side(a.0, a.1, b.0).signum();
        let d4 = side(a.0, a.1, b.1).signum();

        d1 * d2 < 0 && d3 * d4 < 0
    }

    fn inside(bounds: Rectangle<i32>, point: Point<i32>) -> bool {
        (bounds.x..bounds.x + bounds.width).contains(&point.x)
            && (bounds.y..bounds.y + bounds.height).contains(&point.y)
    }

    /// The panes at the showcase's size and a larger one.
    fn panes() -> [Size<i32>; 3] {
        [
            Size::new(395, 332),
            Size::new(715, 472),
            Size::new(300, 260),
        ]
    }

    #[test]
    fn leaders_cross_neither_each_other_nor_labels() {
        for size in panes() {
            let fit = Fit::new(size);
            let notes: Vec<_> = fit.notes.iter().flatten().collect();

            assert_eq!(notes.len(), PARTS.len(), "{size:?}: a callout was dropped");

            for (i, a) in notes.iter().enumerate() {
                for (j, b) in notes.iter().enumerate().skip(i + 1) {
                    assert!(
                        !crosses((a.target, a.elbow), (b.target, b.elbow)),
                        "{size:?}: the leaders of {i} and {j} cross"
                    );
                }

                for (j, b) in notes.iter().enumerate() {
                    let label = b.bounds();
                    let leader = quadrille::draw::shape::line(a.target, a.elbow);

                    assert!(
                        i == j || !leader.iter().any(|&pixel| inside(label, pixel)),
                        "{size:?}: the leader of {i} crosses the label of {j}"
                    );
                }
            }
        }
    }

    #[test]
    fn labels_stay_apart_and_on_the_pane() {
        let fit = Fit::new(Size::new(395, 332));
        let notes: Vec<_> = fit.notes.iter().flatten().collect();

        for (i, a) in notes.iter().enumerate() {
            let bounds = a.bounds();

            assert!(
                bounds.x >= 0 && bounds.x + bounds.width <= fit.size.width,
                "{i}"
            );
            assert!(
                bounds.y >= 0 && bounds.y + bounds.height <= fit.size.height,
                "{i}"
            );

            for b in notes.iter().skip(i + 1) {
                let other = b.bounds();
                let apart = bounds.x + bounds.width <= other.x
                    || other.x + other.width <= bounds.x
                    || bounds.y + bounds.height <= other.y
                    || other.y + other.height <= bounds.y;

                assert!(apart, "{bounds:?} and {other:?} overlap");
            }
        }
    }

    #[test]
    fn every_part_is_found_at_its_target() {
        let fit = Fit::new(Size::new(395, 332));

        assert_eq!(fit.scale, 2);

        for (i, part) in PARTS.iter().enumerate() {
            let target = fit.at(part.target);

            assert!(
                part.shapes.iter().any(|shape| fit.covers(shape, target)),
                "{} is not at its target",
                part.name
            );
            let note = fit.notes[i].expect("every callout fits");

            assert_eq!(fit.hit(note.elbow), Some(i), "{}", part.name);
        }
    }

    #[test]
    fn hover_redraws_only_when_the_part_changes_and_a_click_pins() {
        use canvas::Program;

        let bounds = Rectangle::new(Point::ORIGIN, Size::new(395.0, 332.0));
        let fit = Fit::new(Size::new(395, 332));
        let battery = fit.at((-10, 75));
        let tank = fit.at((4, 93));
        let point = |p: Point<i32>| Point::new(p.x as f32, p.y as f32);
        let moved = |p: Point<i32>| Event::Mouse(mouse::Event::CursorMoved { position: point(p) });
        let pressed = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let at = |p: Point<i32>| mouse::Cursor::Available(point(p));

        let free = Cutaway { pinned: None };
        let mut state = Hover::default();

        let action = free.update(&mut state, &moved(battery), bounds, at(battery));
        assert!(action.is_some());
        assert_eq!(state.part, Some(7));

        let same = battery + iced::Vector::new(3, 1);
        assert!(
            free.update(&mut state, &moved(same), bounds, at(same))
                .is_none()
        );

        let (message, ..) = free
            .update(&mut state, &pressed, bounds, at(battery))
            .expect("a click publishes")
            .into_inner();
        let Some(Message::Pin(Some(pin))) = message else {
            panic!("the battery is pinned");
        };
        assert_eq!(pin.part, 7);

        let pinned = Cutaway { pinned: Some(7) };
        let (message, ..) = pinned
            .update(&mut state, &pressed, bounds, at(battery))
            .expect("a click publishes")
            .into_inner();
        assert!(matches!(message, Some(Message::Pin(None))));

        let _ = pinned.update(&mut state, &moved(tank), bounds, at(tank));
        assert_eq!(state.part, Some(9));
    }

    #[test]
    fn a_pin_names_the_zone_on_the_border() {
        let fit = Fit::new(Size::new(395, 332));
        let (letter, number) = fit.zone(0);

        assert_eq!(letter, 'A');
        assert!((2..=4).contains(&number));
    }

    #[test]
    fn the_signal_goes_round_and_comes_back() {
        let plan = Plan::new(Size::new(214, 219));

        assert!(matches!(plan.route.first(), Some(Stop::Wire(_))));
        assert_eq!(plan.route.last(), Some(&Stop::Radiating));

        for block in [DIPLEXER, LNA, MIXER, DEMOD, PROCESSOR, MODULATOR, AMPLIFIER] {
            assert!(plan.route.contains(&Stop::Block(block)));
        }

        // Consecutive wire stops are neighbouring pixels.
        for pair in plan.route.windows(2) {
            if let [Stop::Wire(a), Stop::Wire(b)] = pair {
                assert!((a.x - b.x).abs() + (a.y - b.y).abs() <= 1, "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn the_diagram_joins_where_branches_meet() {
        let plan = Plan::new(Size::new(214, 219));

        // The oscillator's tee, and the power taps to the amplifier and the
        // modulator.
        assert_eq!(plan.joins.len(), 3);
    }
}
