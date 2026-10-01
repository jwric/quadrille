use std::fmt;
use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicU64, Ordering};

use iced_widget::canvas::{self, Geometry};
use iced_widget::core::{Length, Point, Rectangle, mouse};
use iced_widget::graphics::geometry;

use super::GUTTER;
use crate::canvas::Memo;
use crate::draw::{Anchor, Pen, Raster, rectangle};
use crate::theme::Ramp;
use crate::{Face, Palette, Theme, px, scale};

/// A scrolling record of rows of levels, the newest first: what a
/// [`Waterfall`] draws.
///
/// The rows live in a ring, so adding one costs a copy of the row and
/// nothing else. The caller owns the history and adds a row whenever it has
/// one; a waterfall drawing it rebuilds its picture only when a row has been
/// added since it last drew.
pub struct History {
    bins: usize,
    depth: usize,
    levels: Vec<f32>,
    newest: usize,
    len: usize,
    id: u64,
    revision: u64,
}

/// Tells histories apart, so a waterfall never takes one for another that
/// happens to have had as many rows added.
static NEXT: AtomicU64 = AtomicU64::new(0);

impl History {
    /// An empty history of rows `bins` wide that keeps the newest `depth`.
    pub fn new(bins: usize, depth: usize) -> Self {
        let (bins, depth) = (bins.max(1), depth.max(1));

        Self {
            bins,
            depth,
            levels: vec![f32::NEG_INFINITY; bins * depth],
            newest: depth - 1,
            len: 0,
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            revision: 0,
        }
    }

    /// Adds `row` as the newest, dropping the oldest if the history is full.
    ///
    /// A row of another width is fitted to the history's: each bin takes the
    /// highest level of the row's bins that fall in it.
    pub fn push(&mut self, row: &[f32]) {
        self.newest = (self.newest + 1) % self.depth;
        self.len = (self.len + 1).min(self.depth);
        self.revision += 1;

        let start = self.newest * self.bins;
        let slot = &mut self.levels[start..start + self.bins];

        if row.len() == slot.len() {
            slot.copy_from_slice(row);
        } else {
            for (bin, level) in slot.iter_mut().enumerate() {
                *level = row[scale::bins(bin, self.bins, row.len())]
                    .iter()
                    .fold(f32::NEG_INFINITY, |peak, &level| peak.max(level));
            }
        }
    }

    /// The row added `age` rows ago: 0 is the newest.
    pub fn row(&self, age: usize) -> Option<&[f32]> {
        if age >= self.len {
            return None;
        }

        let index = (self.newest + self.depth - age) % self.depth;

        Some(&self.levels[index * self.bins..(index + 1) * self.bins])
    }

    /// Every row, the newest first.
    pub fn rows(&self) -> impl Iterator<Item = &[f32]> {
        (0..self.len).filter_map(|age| self.row(age))
    }

    /// The peak of each bin over the history, every row lowered by `fall`
    /// for each row added after it: a peak hold that falls `fall` a row.
    pub fn hold(&self, fall: f32) -> Vec<f32> {
        let mut peaks = vec![f32::NEG_INFINITY; self.bins];

        for (age, row) in self.rows().enumerate() {
            let lowered = age as f32 * fall;

            for (peak, level) in peaks.iter_mut().zip(row) {
                *peak = peak.max(level - lowered);
            }
        }

        peaks
    }

    /// The width of a row.
    pub fn bins(&self) -> usize {
        self.bins
    }

    /// How many rows the history keeps.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// How many rows the history holds.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether no row has been added.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// What a picture of the history is drawn from.
    fn version(&self) -> (u64, u64) {
        (self.id, self.revision)
    }
}

impl Clone for History {
    /// A copy with an identity of its own: the two part ways at the next
    /// row.
    fn clone(&self) -> Self {
        Self {
            levels: self.levels.clone(),
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            ..*self
        }
    }
}

impl fmt::Debug for History {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("History")
            .field("bins", &self.bins)
            .field("depth", &self.depth)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

/// A waterfall: a [`History`] drawn as a raster, the newest row at the top,
/// one pixel a row, each level coloured by the step of the theme's [`Ramp`]
/// it falls in.
///
/// The raster is an image at the waterfall's own size, drawn at a whole
/// pixel without filtering, so a bin is a column of flat pixels. It is built
/// again only when the history has had a row added, or the theme or size
/// has changed; drawing the same history again reuses it.
pub struct Waterfall<'a> {
    history: &'a History,
    range: RangeInclusive<f32>,
    gutter: u16,
    rate: Option<f32>,
    width: Length,
    height: Length,
}

/// Creates a [`Waterfall`] of `history`.
pub fn waterfall(history: &History) -> Waterfall<'_> {
    Waterfall {
        history,
        range: -100.0..=0.0,
        gutter: GUTTER,
        rate: None,
        width: Length::Fill,
        height: Length::Fill,
    }
}

impl Waterfall<'_> {
    /// Sets the levels the ramp spans, from the void to its last step.
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    /// Sets the width of the margin left of the raster.
    ///
    /// A [`Spectrum`](super::Spectrum) of the same width and gutter above
    /// the waterfall puts each bin in the same column in both.
    pub fn gutter(mut self, gutter: u16) -> Self {
        self.gutter = gutter;
        self
    }

    /// Marks the age of the rows every second in the gutter, for a history
    /// that has `rows` rows added a second.
    pub fn rate(mut self, rows: f32) -> Self {
        self.rate = (rows > 0.0).then_some(rows);
        self
    }

    /// Sets the width of the [`Waterfall`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Waterfall`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// The frame around the raster, if there is room for one.
    fn frame(&self, width: i32, height: i32) -> Option<Rectangle<i32>> {
        let gutter = i32::from(self.gutter).min(width);
        let frame = rectangle(gutter, 0, width - gutter, height);

        (frame.width >= 3 && frame.height >= 3).then_some(frame)
    }

    /// The raster: one row of pixels a row of history, one column a column.
    fn raster(&self, columns: u32, rows: u32, ramp: &Ramp) -> Raster {
        let (low, high) = (*self.range.start(), *self.range.end());
        let span = if high == low { 1.0 } else { high - low };
        let steps = ramp.steps();
        let mut raster = Raster::new(columns, rows, steps[0]);

        for (age, row) in self.history.rows().take(rows as usize).enumerate() {
            for column in 0..columns {
                let level = row[scale::bins(column as usize, columns as usize, row.len())]
                    .iter()
                    .fold(f32::NEG_INFINITY, |peak, &level| peak.max(level));

                raster.set(column, age as u32, steps[ramp.step((level - low) / span)]);
            }
        }

        raster
    }
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for Waterfall<'_>
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
        let palette = *theme.palette();
        let size = px::floor(bounds.size());

        let Some(frame) = self.frame(size.width, size.height) else {
            return Vec::new();
        };

        let chrome = state.chrome.draw(
            renderer,
            bounds.size(),
            (palette, self.gutter, self.rate),
            |target| {
                let mut pen = Pen::new(target);

                pen.outline(frame, palette.edge);

                if let Some(rate) = self.rate {
                    ages(&mut pen, frame, rate, &palette);
                }
            },
        );

        let picture = (self.history.version(), palette, self.range.clone());

        let raster = state
            .raster
            .draw(renderer, bounds.size(), picture, |target| {
                let (columns, rows) = (frame.width - 2, frame.height - 2);
                let raster = self.raster(columns as u32, rows as u32, &Ramp::of(&palette));

                Pen::new(target).raster(&raster, Point::new(frame.x + 1, frame.y + 1));
            });

        vec![chrome, raster]
    }
}

/// Ticks down the left of `frame` every second of age, and labels on as
/// many of them as fit.
fn ages<Renderer: geometry::Renderer>(
    pen: &mut Pen<'_, Renderer>,
    frame: Rectangle<i32>,
    rate: f32,
    palette: &Palette,
) {
    let face = Face::BODY;
    let (cap, line) = (i32::from(face.cap()), i32::from(face.line()));
    let bottom = frame.y + frame.height - 2;
    let every = scale::step(2.0 * line as f32 / rate, 1).max(1.0) as i32;
    let mut clear = frame.y;

    for second in 1.. {
        let y = frame.y + 1 + (second as f32 * rate).round() as i32;

        if y > bottom {
            break;
        }

        pen.hline(frame.x - 2, frame.x - 1, y, palette.faint);

        let top = y - cap.div_euclid(2);
        let label = format!("{second}s");
        let fits = i32::from(face.width(&label)) + 4 <= frame.x;

        if second % every == 0 && fits && top >= clear && top + cap <= bottom {
            pen.text(
                face,
                label,
                Point::new(frame.x - 4, y),
                Anchor::RIGHT,
                palette.muted,
            );
            clear = top + cap + px::GAP as i32;
        }
    }
}

/// What a raster is drawn from: the history as it stood, the palette and
/// the range of levels.
type Picture = ((u64, u64), Palette, RangeInclusive<f32>);

/// The state of a [`Waterfall`]: its frame and its raster, each drawn again
/// only when what it shows has changed.
pub struct State<Renderer: geometry::Renderer> {
    chrome: Memo<(Palette, u16, Option<f32>), Renderer>,
    raster: Memo<Picture, Renderer>,
}

impl<Renderer: geometry::Renderer> Default for State<Renderer> {
    fn default() -> Self {
        Self {
            chrome: Memo::new(),
            raster: Memo::new(),
        }
    }
}

crate::canvas_widget!(Waterfall<'a>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_history_keeps_its_newest_rows() {
        let mut history = History::new(2, 3);

        for i in 0..5 {
            history.push(&[i as f32, -(i as f32)]);
        }

        assert_eq!(history.len(), 3);
        assert_eq!(history.row(0), Some(&[4.0, -4.0][..]));
        assert_eq!(history.row(2), Some(&[2.0, -2.0][..]));
        assert_eq!(history.row(3), None);
    }

    #[test]
    fn a_row_of_another_width_keeps_its_peaks() {
        let mut history = History::new(2, 1);

        history.push(&[1.0, 5.0, 2.0, 3.0]);

        assert_eq!(history.row(0), Some(&[5.0, 3.0][..]));
    }

    #[test]
    fn a_held_peak_falls_with_age() {
        let mut history = History::new(2, 8);

        history.push(&[-10.0, -50.0]);
        history.push(&[-60.0, -60.0]);
        history.push(&[-60.0, -40.0]);

        assert_eq!(history.hold(5.0), vec![-20.0, -40.0]);
    }

    #[test]
    fn a_copy_is_another_history() {
        let history = History::new(4, 4);

        assert_ne!(history.clone().version(), history.version());
    }
}
