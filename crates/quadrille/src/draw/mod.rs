//! Drawing on a canvas in whole pixels.
//!
//! A [`Pen`] draws onto an iced canvas [`Frame`] with integer coordinates
//! only, so a half pixel cannot be asked for. Shapes are rasterized here, by
//! [`shape`], and handed to the frame as axis-aligned rectangles: they light
//! the same pixels on every renderer.
//!
//! ```no_run
//! # use quadrille::draw::{Anchor, Pen};
//! # use quadrille::{Face, Theme};
//! # use iced::widget::canvas::Frame;
//! # use iced::Point;
//! # fn draw(frame: &mut Frame, theme: &Theme) {
//! let palette = theme.palette();
//! let mut pen = Pen::new(frame);
//!
//! pen.circle(Point::new(40, 40), 30, palette.line);
//! pen.crosshair(Point::new(40, 40), 6, 2, palette.ink);
//! pen.text(Face::BODY, "AUTO", Point::new(40, 76), Anchor::TOP, palette.accent);
//! # }
//! ```
pub mod shape;

mod polygon;
mod raster;
mod sprite;

pub use polygon::Polygon;
pub use raster::Raster;
pub use shape::Direction;
pub use sprite::Sprite;

use std::ops::RangeInclusive;

use iced_widget::canvas::{Frame, Path, Text};
use iced_widget::core::{Color, Point, Rectangle, Size, alignment};
use iced_widget::graphics::geometry;
use iced_widget::text::{Alignment, LineHeight, Shaping};

use crate::Face;

/// Draws on a [`Frame`] in whole pixels.
///
/// Consecutive shapes of one colour are batched into one fill, so a pen
/// should be dropped (or [`flush`](Self::flush)ed) before the frame is used
/// directly.
pub struct Pen<'a, Renderer>
where
    Renderer: geometry::Renderer,
{
    frame: &'a mut Frame<Renderer>,
    color: Color,
    batch: Vec<Rectangle<i32>>,
}

impl<'a, Renderer> Pen<'a, Renderer>
where
    Renderer: geometry::Renderer,
{
    /// Creates a [`Pen`] that draws on `frame`.
    pub fn new(frame: &'a mut Frame<Renderer>) -> Self {
        Self {
            frame,
            color: Color::TRANSPARENT,
            batch: Vec::new(),
        }
    }

    /// Hands every batched shape to the frame.
    pub fn flush(&mut self) {
        if self.batch.is_empty() {
            return;
        }

        let path = Path::new(|builder| {
            for rectangle in self.batch.drain(..) {
                builder.rectangle(
                    Point::new(rectangle.x as f32, rectangle.y as f32),
                    Size::new(rectangle.width as f32, rectangle.height as f32),
                );
            }
        });

        self.frame.fill(&path, self.color);
    }

    /// The frame, with every batched shape drawn.
    pub fn frame(&mut self) -> &mut Frame<Renderer> {
        self.flush();
        self.frame
    }

    /// Fills `rectangle`.
    pub fn fill(&mut self, rectangle: Rectangle<i32>, color: Color) {
        if rectangle.width <= 0 || rectangle.height <= 0 {
            return;
        }

        if color != self.color {
            self.flush();
            self.color = color;
        }

        self.batch.push(rectangle);
    }

    /// Lights one pixel.
    pub fn pixel(&mut self, at: Point<i32>, color: Color) {
        self.fill(rectangle(at.x, at.y, 1, 1), color);
    }

    /// Lights every pixel of `pixels`, merging neighbours into runs.
    pub fn pixels(&mut self, pixels: &[Point<i32>], color: Color) {
        for run in shape::runs(pixels, false) {
            self.fill(run, color);
        }
    }

    /// A horizontal line through row `y`, from column `from` to column `to`,
    /// both included.
    pub fn hline(&mut self, from: i32, to: i32, y: i32, color: Color) {
        let (left, right) = (from.min(to), from.max(to));

        self.fill(rectangle(left, y, right - left + 1, 1), color);
    }

    /// A vertical line down column `x`, from row `from` to row `to`, both
    /// included.
    pub fn vline(&mut self, x: i32, from: i32, to: i32, color: Color) {
        let (top, bottom) = (from.min(to), from.max(to));

        self.fill(rectangle(x, top, 1, bottom - top + 1), color);
    }

    /// A 1 px border just inside `bounds`.
    pub fn outline(&mut self, bounds: Rectangle<i32>, color: Color) {
        let (right, bottom) = (bounds.x + bounds.width - 1, bounds.y + bounds.height - 1);

        if bounds.width <= 0 || bounds.height <= 0 {
            return;
        }

        self.hline(bounds.x, right, bounds.y, color);

        if bounds.height > 1 {
            self.hline(bounds.x, right, bottom, color);
        }

        if bounds.height > 2 {
            self.vline(bounds.x, bounds.y + 1, bottom - 1, color);

            if bounds.width > 1 {
                self.vline(right, bounds.y + 1, bottom - 1, color);
            }
        }
    }

    /// A 1 px line from `from` to `to`, both included.
    pub fn line(&mut self, from: Point<i32>, to: Point<i32>, color: Color) {
        let steep = (to.y - from.y).abs() > (to.x - from.x).abs();

        for run in shape::runs(&shape::line(from, to), steep) {
            self.fill(run, color);
        }
    }

    /// A dashed 1 px line from `from` to `to`, its pattern counted from
    /// `from`.
    ///
    /// Start a dashed line at the end that must look right: the pattern is
    /// whole there and falls where it may at the other end.
    pub fn dashed(&mut self, from: Point<i32>, to: Point<i32>, dash: Dash, color: Color) {
        let pixels: Vec<_> = shape::line(from, to)
            .into_iter()
            .enumerate()
            .filter(|(i, _)| dash.lights(*i as i32))
            .map(|(_, pixel)| pixel)
            .collect();

        let steep = (to.y - from.y).abs() > (to.x - from.x).abs();

        for run in shape::runs(&pixels, steep) {
            self.fill(run, color);
        }
    }

    /// A 1 px polyline through `points`.
    pub fn polyline(&mut self, points: &[Point<i32>], color: Color) {
        self.pixels(&shape::polyline(points), color);
    }

    /// A 1 px curve through points in fractional pixels, each snapped to the
    /// nearest pixel before the segments are drawn.
    pub fn curve(&mut self, points: impl IntoIterator<Item = Point>, color: Color) {
        let mut snapped: Vec<Point<i32>> = Vec::new();

        for point in points {
            let pixel = Point::new(point.x.round() as i32, point.y.round() as i32);

            if snapped.last() != Some(&pixel) {
                snapped.push(pixel);
            }
        }

        self.polyline(&snapped, color);
    }

    /// A 1 px circle of `radius` around the centre pixel.
    pub fn circle(&mut self, centre: Point<i32>, radius: i32, color: Color) {
        self.pixels(&shape::circle(centre, radius), color);
    }

    /// A 1 px circle of `radius` broken into `dash`, with a dash centred
    /// straight up.
    ///
    /// The pattern is stretched to repeat a whole number of times around the
    /// circle, so the last dash meets the first evenly.
    pub fn dashed_circle(&mut self, centre: Point<i32>, radius: i32, dash: Dash, color: Color) {
        self.pixels(&shape::dashed_circle(centre, radius, dash), color);
    }

    /// A filled disc of `radius` around the centre pixel.
    pub fn disc(&mut self, centre: Point<i32>, radius: i32, color: Color) {
        for (y, from, to) in shape::disc(centre, radius) {
            self.hline(from, to, y, color);
        }
    }

    /// A 1 px arc of a circle, clockwise from bearing `from` to bearing `to`.
    ///
    /// Bearings are in degrees clockwise from straight up.
    pub fn arc(&mut self, centre: Point<i32>, radius: i32, from: f32, to: f32, color: Color) {
        self.pixels(&shape::arc(centre, radius, from, to), color);
    }

    /// A ring `weight` pixels wide with its outer edge on the circle of
    /// `radius`.
    pub fn ring(&mut self, centre: Point<i32>, radius: i32, weight: i32, color: Color) {
        self.pixels(&shape::ring(centre, radius, weight), color);
    }

    /// An arc of a ring `weight` pixels wide, clockwise from bearing `from` to
    /// bearing `to`.
    pub fn thick_arc(
        &mut self,
        centre: Point<i32>,
        radius: i32,
        weight: i32,
        from: f32,
        to: f32,
        color: Color,
    ) {
        self.pixels(&shape::thick_arc(centre, radius, weight, from, to), color);
    }

    /// A 1 px axis-aligned ellipse around the centre pixel.
    pub fn ellipse(&mut self, centre: Point<i32>, radius_x: i32, radius_y: i32, color: Color) {
        self.pixels(&shape::ellipse(centre, radius_x, radius_y), color);
    }

    /// A solid 45° arrowhead with its tip at `tip`.
    pub fn arrowhead(&mut self, tip: Point<i32>, towards: Direction, size: i32, color: Color) {
        for strip in shape::triangle(tip, towards, size) {
            self.fill(strip, color);
        }
    }

    /// Four L-shaped marks at the corners of `bounds`, each `arm` pixels long
    /// and `weight` pixels thick: a selection or focus frame that does not
    /// box its contents in.
    pub fn brackets(&mut self, bounds: Rectangle<i32>, arm: i32, weight: i32, color: Color) {
        let (left, top) = (bounds.x, bounds.y);
        let (right, bottom) = (
            bounds.x + bounds.width - weight,
            bounds.y + bounds.height - weight,
        );
        let length = arm.max(weight);

        for (x, y, dx, dy) in [
            (left, top, 1, 1),
            (right, top, -1, 1),
            (left, bottom, 1, -1),
            (right, bottom, -1, -1),
        ] {
            let horizontal_x = if dx > 0 { x } else { x + weight - length };
            let vertical_y = if dy > 0 { y } else { y + weight - length };

            self.fill(rectangle(horizontal_x, y, length, weight), color);
            self.fill(rectangle(x, vertical_y, weight, length), color);
        }
    }

    /// A crosshair on the centre pixel: four arms of `arm` pixels, starting
    /// `gap` pixels out from the centre, which is left dark.
    pub fn crosshair(&mut self, centre: Point<i32>, arm: i32, gap: i32, color: Color) {
        let near = gap + 1;
        let far = gap + arm;

        self.hline(centre.x - far, centre.x - near, centre.y, color);
        self.hline(centre.x + near, centre.x + far, centre.y, color);
        self.vline(centre.x, centre.y - far, centre.y - near, color);
        self.vline(centre.x, centre.y + near, centre.y + far, color);
    }

    /// Fills `bounds` with a 1-bit `pattern`, counted from `origin`.
    ///
    /// Anchor the origin to what the pattern belongs to, not to the screen:
    /// a pattern counted from a panned world's corner moves with the world
    /// instead of shimmering over it.
    pub fn pattern(
        &mut self,
        bounds: Rectangle<i32>,
        pattern: Pattern,
        origin: Point<i32>,
        color: Color,
    ) {
        for y in bounds.y..bounds.y + bounds.height {
            let mut start = None;

            for x in bounds.x..=bounds.x + bounds.width {
                let lit = x < bounds.x + bounds.width && pattern.lights(x - origin.x, y - origin.y);

                match (lit, start) {
                    (true, None) => start = Some(x),
                    (false, Some(first)) => {
                        self.fill(rectangle(first, y, x - first, 1), color);
                        start = None;
                    }
                    _ => {}
                }
            }
        }
    }

    /// A [`Sprite`] with its top-left corner at `at`.
    pub fn sprite(&mut self, sprite: &Sprite, at: Point<i32>, color: Color) {
        for strip in sprite.strips() {
            self.fill(
                rectangle(at.x + strip.x, at.y + strip.y, strip.width, strip.height),
                color,
            );
        }
    }

    /// Sets `content` in `face`, placed by `anchor` at `at`.
    pub fn text(
        &mut self,
        face: Face,
        content: impl Into<String>,
        at: Point<i32>,
        anchor: Anchor,
        color: Color,
    ) {
        let content = content.into();
        let width = i32::from(face.width(&content));

        let x = match anchor.x {
            Horizontal::Left => at.x,
            Horizontal::Centre => at.x - width.div_euclid(2),
            Horizontal::Right => at.x - width,
        };

        let line = i32::from(face.line());

        let y = match anchor.y {
            Vertical::Top => at.y,
            Vertical::CapTop => at.y - i32::from(face.cap_top()),
            Vertical::Middle => {
                at.y - i32::from(face.cap_top()) - i32::from(face.cap()).div_euclid(2)
            }
            Vertical::Baseline => at.y - i32::from(face.baseline()),
            Vertical::Bottom => at.y - line,
        };

        self.flush();
        self.frame.fill_text(Text {
            content,
            position: Point::new(x as f32, y as f32),
            max_width: f32::INFINITY,
            color,
            size: f32::from(face.size()).into(),
            line_height: LineHeight::Absolute(f32::from(face.line()).into()),
            font: face.font,
            align_x: Alignment::Left,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::Basic,
            wrapping: iced_widget::core::text::Wrapping::None,
            ellipsis: iced_widget::core::text::Ellipsis::None,
        });
    }

    /// Sets `content` struck through: a bar across the middle of its
    /// capitals, reaching half a cell past each end.
    pub fn strike(
        &mut self,
        face: Face,
        content: &str,
        at: Point<i32>,
        anchor: Anchor,
        color: Color,
    ) {
        let width = i32::from(face.width(content));
        let cap = i32::from(face.cap());
        let left = match anchor.x {
            Horizontal::Left => at.x,
            Horizontal::Centre => at.x - width.div_euclid(2),
            Horizontal::Right => at.x - width,
        };
        let cap_top = match anchor.y {
            Vertical::Top => at.y + i32::from(face.cap_top()),
            Vertical::CapTop => at.y,
            Vertical::Middle => at.y - cap.div_euclid(2),
            Vertical::Baseline => at.y - cap,
            Vertical::Bottom => at.y - i32::from(face.line()) + i32::from(face.cap_top()),
        };

        let weight = (cap / 4).max(1);
        let reach = i32::from(face.advance()) / 2;

        self.text(
            face,
            content,
            Point::new(left, cap_top),
            Anchor::new(Horizontal::Left, Vertical::CapTop),
            color,
        );
        self.fill(
            rectangle(
                left - reach,
                cap_top + (cap - weight).div_euclid(2),
                width + 2 * reach,
                weight,
            ),
            color,
        );
    }

    /// The marks of a [`Ticks`] scale.
    pub fn ticks(&mut self, ticks: &Ticks, color: Color) {
        if ticks.step <= 0 || ticks.length <= 0 {
            return;
        }

        let (near, far) = match ticks.towards {
            Direction::Up | Direction::Left => (ticks.edge - ticks.length + 1, ticks.edge),
            Direction::Down | Direction::Right => (ticks.edge, ticks.edge + ticks.length - 1),
        };

        let mut position = *ticks.span.start();

        while position <= *ticks.span.end() {
            match ticks.along {
                Axis::Horizontal => self.vline(position, near, far, color),
                Axis::Vertical => self.hline(near, far, position, color),
            }

            position += ticks.step;
        }
    }

    /// The top edge of a group frame: a rule across `bounds` with the ends
    /// dropping `drop` pixels, broken in the middle for `label`.
    ///
    /// The rule runs through the middle of the label's capitals, and `bounds`
    /// is the label's line: the group's contents start below it.
    pub fn group_rule(
        &mut self,
        bounds: Rectangle<i32>,
        drop: i32,
        label: Lettering<'_>,
        line: Color,
    ) {
        let face = label.face;
        let y = bounds.y + i32::from(face.cap_top()) + i32::from(face.cap()).div_euclid(2);
        let (left, right) = (bounds.x, bounds.x + bounds.width - 1);
        let width = i32::from(face.width(label.text));
        let gap = i32::from(face.advance());
        let label_left = left + (bounds.width - width).div_euclid(2);

        if width > 0 && label_left - gap > left && label_left + width + gap <= right {
            self.hline(left, label_left - gap, y, line);
            self.hline(label_left + width + gap - 1, right, y, line);
            self.letter(label, Point::new(label_left, bounds.y), Anchor::TOP_LEFT);
        } else {
            self.hline(left, right, y, line);
        }

        self.vline(left, y + 1, y + drop, line);
        self.vline(right, y + 1, y + drop, line);
    }

    fn letter(&mut self, label: Lettering<'_>, at: Point<i32>, anchor: Anchor) {
        self.text(label.face, label.text, at, anchor, label.color);
    }
}

/// Text for an annotation: what it says, in which face and colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lettering<'a> {
    /// The face it is set in.
    pub face: Face,
    /// What it says.
    pub text: &'a str,
    /// Its colour.
    pub color: Color,
}

impl<'a> Lettering<'a> {
    /// `text` in `face` and `color`.
    pub fn new(face: Face, text: &'a str, color: Color) -> Self {
        Self { face, text, color }
    }
}

/// A row of ticks along one edge of a scale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticks {
    /// Whether the ticks stand along a row or down a column.
    pub along: Axis,
    /// The row or column the ticks start from.
    pub edge: i32,
    /// The first and last positions a tick may take.
    pub span: RangeInclusive<i32>,
    /// The pixels from one tick to the next.
    pub step: i32,
    /// How long each tick is.
    pub length: i32,
    /// Which way the ticks reach from the edge.
    pub towards: Direction,
}

impl<Renderer> Drop for Pen<'_, Renderer>
where
    Renderer: geometry::Renderer,
{
    fn drop(&mut self) {
        self.flush();
    }
}

/// A rectangle in whole pixels.
pub const fn rectangle(x: i32, y: i32, width: i32, height: i32) -> Rectangle<i32> {
    Rectangle {
        x,
        y,
        width,
        height,
    }
}

/// A row or a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Along a row.
    Horizontal,
    /// Down a column.
    Vertical,
}

/// A dash pattern: `on` pixels lit, then `off` pixels dark, shifted by
/// `phase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dash {
    /// Pixels lit per period.
    pub on: u8,
    /// Pixels dark per period.
    pub off: u8,
    /// Pixels into the period the pattern starts.
    pub phase: u8,
}

impl Dash {
    /// Long dashes for hidden edges: six on, six off.
    pub const HIDDEN: Self = Self::new(6, 6);

    /// Short dashes: two on, two off.
    pub const SHORT: Self = Self::new(2, 2);

    /// A dot every other pixel.
    pub const DOTTED: Self = Self::new(1, 1);

    /// A dot every third pixel.
    pub const SPARSE: Self = Self::new(1, 2);

    /// A pattern of `on` lit and `off` dark pixels.
    pub const fn new(on: u8, off: u8) -> Self {
        Self { on, off, phase: 0 }
    }

    /// The same pattern shifted by `phase` pixels.
    pub const fn phase(self, phase: u8) -> Self {
        Self { phase, ..self }
    }

    /// Whether the `index`th pixel of a line is lit.
    pub fn lights(self, index: i32) -> bool {
        let period = i32::from(self.on) + i32::from(self.off);

        period == 0 || (index + i32::from(self.phase)).rem_euclid(period) < i32::from(self.on)
    }
}

/// A 1-bit fill pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// Every pixel.
    Solid,
    /// Every other pixel, alternating by row.
    Checker,
    /// A 4 × 4 ordered dither with `n` of every 16 pixels lit, from 0 to 16.
    Bayer(u8),
    /// A dot every `n` pixels along both axes.
    Grid(u8),
    /// Diagonal lines every `n` pixels, rising to the right.
    Hatch(u8),
    /// Every `n`th row.
    Rows(u8),
    /// Every `n`th column.
    Columns(u8),
}

impl Pattern {
    /// Whether the pixel at `(x, y)` from the pattern's origin is lit.
    pub fn lights(self, x: i32, y: i32) -> bool {
        const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

        let every = |n: u8, v: i32| v.rem_euclid(i32::from(n.max(1))) == 0;

        match self {
            Self::Solid => true,
            Self::Checker => (x + y).rem_euclid(2) == 0,
            Self::Bayer(level) => BAYER[y.rem_euclid(4) as usize][x.rem_euclid(4) as usize] < level,
            Self::Grid(n) => every(n, x) && every(n, y),
            Self::Hatch(n) => every(n, x + y),
            Self::Rows(n) => every(n, y),
            Self::Columns(n) => every(n, x),
        }
    }
}

/// Where a piece of text is placed relative to the point it is drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchor {
    /// Which part of the text's width is at the point.
    pub x: Horizontal,
    /// Which part of the text's line is at the point.
    pub y: Vertical,
}

impl Anchor {
    /// The top-left corner of the line box.
    pub const TOP_LEFT: Self = Self::new(Horizontal::Left, Vertical::Top);
    /// The middle of the top of the line box.
    pub const TOP: Self = Self::new(Horizontal::Centre, Vertical::Top);
    /// The middle of the baseline.
    pub const BASELINE: Self = Self::new(Horizontal::Centre, Vertical::Baseline);
    /// The left end of the baseline.
    pub const BASELINE_LEFT: Self = Self::new(Horizontal::Left, Vertical::Baseline);
    /// The right end of the baseline.
    pub const BASELINE_RIGHT: Self = Self::new(Horizontal::Right, Vertical::Baseline);
    /// The middle of the capitals, horizontally and vertically.
    pub const CENTRE: Self = Self::new(Horizontal::Centre, Vertical::Middle);
    /// The left end of the middle of the capitals.
    pub const LEFT: Self = Self::new(Horizontal::Left, Vertical::Middle);
    /// The right end of the middle of the capitals.
    pub const RIGHT: Self = Self::new(Horizontal::Right, Vertical::Middle);

    /// An anchor from its two parts.
    pub const fn new(x: Horizontal, y: Vertical) -> Self {
        Self { x, y }
    }
}

/// A horizontal [`Anchor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Horizontal {
    /// The first column of the text.
    Left,
    /// The middle column of the text, rounded left.
    Centre,
    /// One past the last column of the text.
    Right,
}

/// A vertical [`Anchor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// The top of the line box.
    Top,
    /// The top row of the capitals.
    CapTop,
    /// The middle row of the capitals, rounded up.
    Middle,
    /// The row just below the capitals.
    Baseline,
    /// The bottom of the line box.
    Bottom,
}
