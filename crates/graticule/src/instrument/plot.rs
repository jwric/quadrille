use std::borrow::Cow;
use std::cell::RefCell;

use iced_widget::canvas::{self, Cache, Frame, Geometry};
use iced_widget::core::{Color, Element, Length, Point, Rectangle, mouse};
use iced_widget::graphics::geometry;

use super::keep;
use super::scale::{self, End};
use crate::draw::{Anchor, Dash, Direction, Pen, rectangle};
use crate::{Face, Palette, Theme, px};

/// Minor ticks per division.
const MINOR: i32 = 5;
/// Room either side of the graticule for the ground and trigger pointers.
const MARGIN: i32 = 6;
/// The size of a pointer, from its tip to its base.
const POINTER: i32 = 2;

/// What one division is worth, and in what unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    /// The value of one division.
    pub per_division: f32,
    /// The unit, without a prefix: `V`, `s`.
    pub unit: &'static str,
}

impl Scale {
    /// `per_division` of `unit` a division.
    pub const fn new(per_division: f32, unit: &'static str) -> Self {
        Self { per_division, unit }
    }
}

/// One channel of a [`Plot`]: samples spread evenly across the graticule,
/// the first on its left edge and the last on its right.
pub struct Trace<'a> {
    samples: Cow<'a, [f32]>,
    scale: Scale,
    offset: f32,
    label: Option<String>,
    color: Option<fn(&Palette) -> Color>,
}

/// Creates a [`Trace`] of `samples`, one unit a division.
pub fn trace<'a>(samples: impl Into<Cow<'a, [f32]>>) -> Trace<'a> {
    Trace {
        samples: samples.into(),
        scale: Scale::new(1.0, ""),
        offset: 0.0,
        label: None,
        color: None,
    }
}

impl Trace<'_> {
    /// Sets what a division of the trace is worth.
    pub fn scale(mut self, per_division: f32, unit: &'static str) -> Self {
        self.scale = Scale::new(per_division, unit);
        self
    }

    /// Moves the trace's zero `divisions` above the centre line.
    pub fn offset(mut self, divisions: f32) -> Self {
        self.offset = divisions;
        self
    }

    /// Names the trace in the plot's legend.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the colour of the trace, in place of its [`channel`] colour.
    pub fn color(mut self, color: fn(&Palette) -> Color) -> Self {
        self.color = Some(color);
        self
    }

    fn tone(&self, palette: &Palette, index: usize) -> Color {
        self.color
            .map_or_else(|| channel(palette, index), |color| color(palette))
    }
}

/// The colour of the `index`th channel.
///
/// Channels take the live colour, then the accent, caution and ink, skipping
/// any the palette has already given: on a monochrome display, where the
/// accent is the live colour, the second channel still stands apart.
pub fn channel(palette: &Palette, index: usize) -> Color {
    let mut tones: Vec<Color> = Vec::with_capacity(5);

    for tone in [
        palette.live,
        palette.accent,
        palette.caution,
        palette.ink,
        palette.muted,
    ] {
        if !tones.contains(&tone) {
            tones.push(tone);
        }
    }

    tones[index % tones.len()]
}

/// A pair of measuring cursors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cursors {
    /// Two vertical lines, in divisions from the left edge, measuring in
    /// the plot's [`horizontal`](Plot::horizontal) units.
    Vertical(f32, f32),
    /// Two horizontal lines, in divisions above the centre line, measuring
    /// in the first trace's units.
    Horizontal(f32, f32),
}

/// An oscilloscope screen: a graticule of divisions and the traces over it.
///
/// Each trace is decimated to one run of pixels per column holding the
/// lowest and highest sample that land there, so a spike narrower than a
/// column still reaches its peak. Legends and readouts sit above and below
/// the graticule when there is room, and are left off, never cut, when there
/// is not.
pub struct Plot<'a> {
    traces: Vec<Trace<'a>>,
    columns: u16,
    rows: u16,
    horizontal: Option<Scale>,
    trigger: Option<(f32, f32)>,
    cursors: Option<Cursors>,
    width: Length,
    height: Length,
}

/// Creates an empty [`Plot`] of 10 × 8 divisions.
pub fn plot<'a>() -> Plot<'a> {
    Plot {
        traces: Vec::new(),
        columns: 10,
        rows: 8,
        horizontal: None,
        trigger: None,
        cursors: None,
        width: Length::Fill,
        height: Length::Fill,
    }
}

impl<'a> Plot<'a> {
    /// Adds a [`Trace`].
    pub fn push(mut self, trace: Trace<'a>) -> Self {
        self.traces.push(trace);
        self
    }

    /// Sets the number of divisions across and down.
    pub fn divisions(mut self, columns: u16, rows: u16) -> Self {
        self.columns = columns.max(1);
        self.rows = rows.max(1);
        self
    }

    /// Sets what a division across is worth, for the legend and the cursors.
    pub fn horizontal(mut self, per_division: f32, unit: &'static str) -> Self {
        self.horizontal = Some(Scale::new(per_division, unit));
        self
    }

    /// Marks the trigger: `at` divisions from the left edge, at `level` in
    /// the first trace's units.
    pub fn trigger(mut self, at: f32, level: f32) -> Self {
        self.trigger = Some((at, level));
        self
    }

    /// Shows a pair of [`Cursors`] and what lies between them.
    pub fn cursors(mut self, cursors: Cursors) -> Self {
        self.cursors = Some(cursors);
        self
    }

    /// Sets the width of the [`Plot`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Plot`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    fn screen(&self, width: i32, height: i32) -> Option<Screen> {
        Screen::fit(width, height, i32::from(self.columns), i32::from(self.rows))
    }
}

/// Where the parts of a [`Plot`] go in its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Screen {
    grid: Rectangle<i32>,
    division: i32,
    labelled: bool,
}

impl Screen {
    /// The largest graticule of `columns` × `rows` square divisions that fits
    /// `width` × `height`, with a line of labels above and below if they fit
    /// too.
    ///
    /// A division five pixels wide or more is a multiple of five, so every
    /// minor tick lands on a whole pixel.
    fn fit(width: i32, height: i32, columns: i32, rows: i32) -> Option<Self> {
        let strip = i32::from(Face::BODY.line()) + px::TIGHT as i32;
        let margin = if width > 2 * MARGIN + columns * MINOR {
            MARGIN
        } else {
            0
        };

        let division = |room_x: i32, room_y: i32| {
            let fit = ((room_x - 1) / columns).min((room_y - 1) / rows);

            if fit >= MINOR { fit - fit % MINOR } else { fit }
        };

        let labelled = division(width - 2 * margin, height - 2 * strip);
        let (division, labelled) = if labelled >= MINOR {
            (labelled, true)
        } else {
            (division(width - 2 * margin, height), false)
        };

        if division < 2 {
            return None;
        }

        let (grid_width, grid_height) = (columns * division + 1, rows * division + 1);

        Some(Self {
            grid: rectangle(
                px::centre(width, grid_width),
                if labelled { strip } else { 0 },
                grid_width,
                grid_height,
            ),
            division,
            labelled,
        })
    }

    fn right(&self) -> i32 {
        self.grid.x + self.grid.width - 1
    }

    fn bottom(&self) -> i32 {
        self.grid.y + self.grid.height - 1
    }

    /// The row of the centre line.
    fn middle(&self) -> i32 {
        self.grid.y + (self.grid.height - 1) / 2
    }

    /// The row `divisions` above the centre line.
    fn row(&self, divisions: f32) -> i32 {
        self.middle() - (divisions * self.division as f32).round() as i32
    }

    /// The column `divisions` from the left edge.
    fn column(&self, divisions: f32) -> i32 {
        self.grid.x + (divisions * self.division as f32).round() as i32
    }
}

/// The marks inside a graticule's frame.
#[derive(Debug, Default)]
struct Marks {
    /// A dot every minor tick along each division line.
    dots: Vec<Point<i32>>,
    /// A tick three pixels long across the centre lines every minor tick.
    ticks: Vec<Point<i32>>,
}

impl Marks {
    fn of(screen: &Screen, columns: i32, rows: i32) -> Self {
        let Screen { grid, division, .. } = *screen;
        let (right, bottom) = (screen.right(), screen.bottom());
        let minor = if division % MINOR == 0 {
            division / MINOR
        } else {
            2
        };
        let down = |x: i32| {
            (grid.y + minor..bottom)
                .step_by(minor as usize)
                .map(move |y| (x, y))
        };
        let across = |y: i32| {
            (grid.x + minor..right)
                .step_by(minor as usize)
                .map(move |x| (x, y))
        };

        let mut marks = Self::default();

        for k in 1..columns {
            marks
                .dots
                .extend(down(grid.x + k * division).map(|(x, y)| Point::new(x, y)));
        }

        for k in 1..rows {
            marks
                .dots
                .extend(across(grid.y + k * division).map(|(x, y)| Point::new(x, y)));
        }

        // A tick every two pixels would fill the line in.
        if division % MINOR == 0 && minor >= 3 {
            if columns % 2 == 0 {
                for (x, y) in down(grid.x + columns / 2 * division) {
                    marks.ticks.extend([x - 1, x + 1].map(|x| Point::new(x, y)));
                }
            }

            if rows % 2 == 0 {
                for (x, y) in across(grid.y + rows / 2 * division) {
                    marks.ticks.extend([y - 1, y + 1].map(|y| Point::new(x, y)));
                }
            }
        }

        marks
    }
}

/// Draws a graticule: the glass, the frame, dotted division lines, and ticks
/// across the centre lines.
fn graticule<Renderer: geometry::Renderer>(
    pen: &mut Pen<'_, Renderer>,
    screen: &Screen,
    columns: i32,
    rows: i32,
    palette: &Palette,
) {
    let marks = Marks::of(screen, columns, rows);

    pen.fill(screen.grid, palette.void);
    pen.pixels(&marks.dots, palette.faint);
    pen.pixels(&marks.ticks, palette.line);
    pen.outline(screen.grid, palette.edge);
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for Plot<'_>
where
    Renderer: geometry::Renderer + 'static,
{
    type State = State<Renderer>;

    fn draw(
        &self,
        state: &State<Renderer>,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let palette = theme.palette();
        let size = px::floor(bounds.size());

        let Some(screen) = self.screen(size.width, size.height) else {
            return Vec::new();
        };

        let (columns, rows) = (i32::from(self.columns), i32::from(self.rows));

        keep(
            &state.chrome,
            &state.drawn,
            (self.columns, self.rows, *palette),
        );

        let chrome = state.chrome.draw(renderer, bounds.size(), |frame| {
            graticule(&mut Pen::new(frame), &screen, columns, rows, palette);
        });

        let mut frame = Frame::new(renderer, bounds.size());

        {
            let mut pen = Pen::new(&mut frame);

            self.draw_traces(&mut pen, &screen, palette);
            self.draw_cursors(&mut pen, &screen, palette);
            self.draw_pointers(&mut pen, &screen, palette);
            self.draw_labels(&mut pen, &screen, palette);
        }

        vec![chrome, frame.into_geometry()]
    }
}

impl Plot<'_> {
    fn draw_traces<Renderer: geometry::Renderer>(
        &self,
        pen: &mut Pen<'_, Renderer>,
        screen: &Screen,
        palette: &Palette,
    ) {
        let grid = screen.grid;

        for (index, trace) in self.traces.iter().enumerate() {
            let color = trace.tone(palette, index);
            let per_division = trace.scale.per_division;
            let row = |value: f32| screen.row(value / per_division + trace.offset);

            let mut runs: Vec<(i32, i32)> = scale::extremes(&trace.samples, grid.width as usize)
                .into_iter()
                .map(|(low, high)| (row(high), row(low)))
                .collect();

            scale::bridge(&mut runs);

            for (column, (top, bottom)) in runs.into_iter().enumerate() {
                let (top, bottom) = (top.max(grid.y), bottom.min(screen.bottom()));

                if top <= bottom {
                    pen.vline(grid.x + column as i32, top, bottom, color);
                }
            }
        }
    }

    fn draw_pointers<Renderer: geometry::Renderer>(
        &self,
        pen: &mut Pen<'_, Renderer>,
        screen: &Screen,
        palette: &Palette,
    ) {
        let grid = screen.grid;
        let rows = grid.y..=screen.bottom();

        if grid.x >= MARGIN {
            for (index, trace) in self.traces.iter().enumerate() {
                let y = screen.row(trace.offset);

                if rows.contains(&y) {
                    pen.arrowhead(
                        Point::new(grid.x - 2, y),
                        Direction::Right,
                        POINTER,
                        trace.tone(palette, index),
                    );
                }
            }
        }

        let Some((at, level)) = self.trigger else {
            return;
        };

        if let Some(source) = self.traces.first() {
            let y = screen.row(level / source.scale.per_division + source.offset);

            if rows.contains(&y) && grid.x >= MARGIN {
                pen.arrowhead(
                    Point::new(screen.right() + 2, y),
                    Direction::Left,
                    POINTER,
                    palette.accent,
                );
            }
        }

        let x = screen.column(at);

        if (grid.x..=screen.right()).contains(&x) {
            pen.arrowhead(
                Point::new(x, grid.y + POINTER),
                Direction::Down,
                POINTER,
                palette.accent,
            );
        }
    }

    fn draw_cursors<Renderer: geometry::Renderer>(
        &self,
        pen: &mut Pen<'_, Renderer>,
        screen: &Screen,
        palette: &Palette,
    ) {
        let grid = screen.grid;

        match self.cursors {
            Some(Cursors::Vertical(from, to)) => {
                for at in [from, to] {
                    let x = screen.column(at);

                    if (grid.x..=screen.right()).contains(&x) {
                        pen.dashed(
                            Point::new(x, grid.y + 1),
                            Point::new(x, screen.bottom() - 1),
                            Dash::SHORT,
                            palette.ink,
                        );
                    }
                }
            }
            Some(Cursors::Horizontal(from, to)) => {
                for at in [from, to] {
                    let y = screen.row(at);

                    if (grid.y..=screen.bottom()).contains(&y) {
                        pen.dashed(
                            Point::new(grid.x + 1, y),
                            Point::new(screen.right() - 1, y),
                            Dash::SHORT,
                            palette.ink,
                        );
                    }
                }
            }
            None => {}
        }
    }

    /// The cursors' readout: the distance between them, and its reciprocal
    /// when that is a time.
    fn readout(&self) -> Option<String> {
        match self.cursors? {
            Cursors::Vertical(from, to) => {
                let scale = self.horizontal?;
                let delta = (to - from).abs() * scale.per_division;
                let mut readout = format!("Δ {}", scale::engineering(delta, scale.unit));

                if scale.unit == "s" && delta > 0.0 {
                    readout += &format!("  1/Δ {}", scale::engineering(1.0 / delta, "Hz"));
                }

                Some(readout)
            }
            Cursors::Horizontal(from, to) => {
                let scale = self.traces.first()?.scale;
                let delta = (to - from).abs() * scale.per_division;

                Some(format!("Δ {}", scale::engineering(delta, scale.unit)))
            }
        }
    }

    fn draw_labels<Renderer: geometry::Renderer>(
        &self,
        pen: &mut Pen<'_, Renderer>,
        screen: &Screen,
        palette: &Palette,
    ) {
        if !screen.labelled {
            return;
        }

        let face = Face::BODY;
        let grid = screen.grid;
        let span = grid.x..=screen.right();
        let gap = i32::from(face.advance());
        let width = |text: &str| i32::from(face.width(text));

        // Above: the trigger level from the left, the cursors' readout from
        // the right.
        let mut above: Vec<(String, End, Color)> = Vec::new();

        if let (Some((_, level)), Some(source)) = (self.trigger, self.traces.first()) {
            above.push((
                format!("T {}", scale::engineering(level, source.scale.unit)),
                End::Start,
                palette.accent,
            ));
        }

        if let Some(readout) = self.readout() {
            above.push((readout, End::End, palette.ink));
        }

        let top = grid.y - px::TIGHT as i32 - i32::from(face.line());
        let row: Vec<_> = above.iter().map(|(t, end, _)| (width(t), *end)).collect();

        for ((text, _, color), left) in above.iter().zip(scale::row(&row, span.clone(), gap)) {
            if let Some(left) = left {
                pen.text(face, text, Point::new(left, top), Anchor::TOP_LEFT, *color);
            }
        }

        // Below: each trace's scale from the left, the horizontal scale from
        // the right.
        let mut below: Vec<(String, End, Color)> = Vec::new();

        if let Some(scale) = self.horizontal {
            below.push((
                format!("{}/DIV", scale::engineering(scale.per_division, scale.unit)),
                End::End,
                palette.muted,
            ));
        }

        for (index, trace) in self.traces.iter().enumerate() {
            let value = scale::engineering(trace.scale.per_division, trace.scale.unit);
            let text = match &trace.label {
                Some(label) => format!("{label} {value}"),
                None => value,
            };

            below.push((text, End::Start, trace.tone(palette, index)));
        }

        let top = screen.bottom() + 1 + px::TIGHT as i32;
        let row: Vec<_> = below.iter().map(|(t, end, _)| (width(t), *end)).collect();

        for ((text, _, color), left) in below.iter().zip(scale::row(&row, span, gap)) {
            if let Some(left) = left {
                pen.text(face, text, Point::new(left, top), Anchor::TOP_LEFT, *color);
            }
        }
    }
}

/// The state of a [`Plot`]: its graticule, drawn once for each size and
/// theme.
pub struct State<Renderer: geometry::Renderer> {
    chrome: Cache<Renderer>,
    drawn: RefCell<Option<(u16, u16, Palette)>>,
}

impl<Renderer: geometry::Renderer> Default for State<Renderer> {
    fn default() -> Self {
        Self {
            chrome: Cache::new(),
            drawn: RefCell::new(None),
        }
    }
}

impl<'a, Message, Renderer> From<Plot<'a>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + 'static,
{
    fn from(plot: Plot<'a>) -> Self {
        let (width, height) = (plot.width, plot.height);

        iced_widget::canvas(plot).width(width).height(height).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divisions_are_whole_minor_ticks() {
        let screen = Screen::fit(263, 229, 10, 8).unwrap();

        assert_eq!(screen.division, 25);
        assert_eq!(screen.grid, rectangle(6, 14, 251, 201));
        assert!(screen.labelled);
        assert_eq!(screen.middle(), 114);
    }

    #[test]
    fn labels_give_way_to_the_graticule() {
        let screen = Screen::fit(263, 60, 10, 8).unwrap();

        assert!(!screen.labelled);
        assert_eq!(screen.division, 5);
        assert_eq!(screen.grid.y, 0);
    }

    #[test]
    fn a_graticule_dots_its_divisions_and_ticks_its_centre_lines() {
        let screen = Screen {
            grid: rectangle(0, 0, 31, 31),
            division: 15,
            labelled: false,
        };
        let marks = Marks::of(&screen, 2, 2);

        let art: Vec<String> = (0..31)
            .map(|y| {
                (0..31)
                    .map(|x| {
                        let at = Point::new(x, y);

                        if x == 0 || y == 0 || x == 30 || y == 30 {
                            '#'
                        } else if marks.ticks.contains(&at) {
                            '+'
                        } else if marks.dots.contains(&at) {
                            'o'
                        } else {
                            '.'
                        }
                    })
                    .collect()
            })
            .collect();

        let expected = [
            "###############################",
            "#.............................#",
            "#.............................#",
            "#.............+o+.............#",
            "#.............................#",
            "#.............................#",
            "#.............+o+.............#",
            "#.............................#",
            "#.............................#",
            "#.............+o+.............#",
            "#.............................#",
            "#.............................#",
            "#.............+o+.............#",
            "#.............................#",
            "#..+..+..+..+..+..+..+..+..+..#",
            "#..o..o..o..o.+o+.o..o..o..o..#",
            "#..+..+..+..+..+..+..+..+..+..#",
            "#.............................#",
            "#.............+o+.............#",
            "#.............................#",
            "#.............................#",
            "#.............+o+.............#",
            "#.............................#",
            "#.............................#",
            "#.............+o+.............#",
            "#.............................#",
            "#.............................#",
            "#.............+o+.............#",
            "#.............................#",
            "#.............................#",
            "###############################",
        ];

        assert_eq!(art, expected, "\n{}\n", art.join("\n"));
    }

    #[test]
    fn a_plot_too_small_to_draw_is_left_blank() {
        assert_eq!(Screen::fit(12, 12, 10, 8), None);
    }

    #[test]
    fn channels_stay_apart_on_a_monochrome_display() {
        let lcd = Palette::LCD;

        assert_ne!(channel(&lcd, 0), channel(&lcd, 1));
        assert_eq!(channel(&Palette::TERMINAL, 1), Palette::TERMINAL.accent);
    }
}
