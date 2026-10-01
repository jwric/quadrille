//! Technical drawing: the line types, notes, dimensions, wiring and sheet of
//! an engineering drawing, as verbs of the toolkit's [`Pen`].
//!
//! Only the drawing page draws in this hand, so it is the console's own
//! rather than the toolkit's. It reaches the pen the way any application or
//! kit would: [`Drafting`] adds its verbs to [`Pen`], and draws with nothing
//! but the pen's public strokes.
mod dimension;
mod linetype;
mod note;
mod sheet;
mod wire;

pub use linetype::Chain;
pub use note::{Note, fan};
pub use sheet::{Field, Sheet, Table};
pub use wire::{Wire, junctions};

use std::ops::RangeInclusive;

use iced::advanced::graphics::geometry;
use iced::{Color, Point};
use quadrille::draw::{Lettering, Pen};

/// The verbs of technical drawing, for a [`Pen`].
pub trait Drafting {
    /// A 1 px chain line from `from` to `to`, its pattern centred on the
    /// middle pixel so that both ends look alike.
    ///
    /// The middle pixel is the middle of a long dash; a line an odd number of
    /// pixels long with an odd `long` is exactly symmetric.
    fn chain(&mut self, from: Point<i32>, to: Point<i32>, chain: Chain, color: Color);

    /// A callout: a leader from `target` to `elbow`, then a shelf along the
    /// elbow's row with `label` set on it.
    ///
    /// The shelf runs away from the target: right when the elbow is right of
    /// it, left otherwise. It overhangs the label by two pixels at each end.
    fn callout(&mut self, target: Point<i32>, elbow: Point<i32>, label: Lettering<'_>, line: Color);

    /// A [`Note`]: its leader, shelves and drop in `line`, and its label in
    /// the label's colour.
    fn note(&mut self, note: &Note<'_>, line: Color);

    /// A horizontal dimension along row `y` across the columns of `span`: end
    /// ticks, and the line broken around `label` in the middle.
    fn dimension(&mut self, span: RangeInclusive<i32>, y: i32, label: Lettering<'_>, line: Color);

    /// A vertical dimension down column `x` across the rows of `span`: end
    /// ticks, and the line broken around `label`, which is set level across
    /// the middle.
    fn vertical_dimension(
        &mut self,
        x: i32,
        span: RangeInclusive<i32>,
        label: Lettering<'_>,
        line: Color,
    );

    /// A 1 px [`Wire`].
    fn wire(&mut self, wire: &Wire, color: Color);

    /// A junction dot: a 3 × 3 square on the centre pixel.
    fn junction(&mut self, at: Point<i32>, color: Color);

    /// The border of `sheet`: trim and border lines, zone divisions and
    /// centring marks in `line`, zone numbers and letters in `marks`.
    ///
    /// A centring mark crosses the middle of each side and reaches a few
    /// pixels into the drawing.
    fn sheet(&mut self, sheet: &Sheet, line: Color, marks: Color);

    /// `table` with its top-left corner at `at`, `width` pixels wide: rules
    /// in `line`, names in `name` and values in `value`.
    fn table(
        &mut self,
        table: &Table<'_>,
        at: Point<i32>,
        width: i32,
        line: Color,
        name: Color,
        value: Color,
    );
}

impl<Renderer> Drafting for Pen<'_, Renderer>
where
    Renderer: geometry::Renderer,
{
    fn chain(&mut self, from: Point<i32>, to: Point<i32>, chain: Chain, color: Color) {
        linetype::chain(self, from, to, chain, color);
    }

    fn callout(
        &mut self,
        target: Point<i32>,
        elbow: Point<i32>,
        label: Lettering<'_>,
        line: Color,
    ) {
        note::callout(self, target, elbow, label, line);
    }

    fn note(&mut self, note: &Note<'_>, line: Color) {
        note::note(self, note, line);
    }

    fn dimension(&mut self, span: RangeInclusive<i32>, y: i32, label: Lettering<'_>, line: Color) {
        dimension::dimension(self, span, y, label, line);
    }

    fn vertical_dimension(
        &mut self,
        x: i32,
        span: RangeInclusive<i32>,
        label: Lettering<'_>,
        line: Color,
    ) {
        dimension::vertical_dimension(self, x, span, label, line);
    }

    fn wire(&mut self, wire: &Wire, color: Color) {
        wire::wire(self, wire, color);
    }

    fn junction(&mut self, at: Point<i32>, color: Color) {
        wire::junction(self, at, color);
    }

    fn sheet(&mut self, sheet: &Sheet, line: Color, marks: Color) {
        sheet::sheet(self, sheet, line, marks);
    }

    fn table(
        &mut self,
        table: &Table<'_>,
        at: Point<i32>,
        width: i32,
        line: Color,
        name: Color,
        value: Color,
    ) {
        sheet::table(self, table, at, width, line, name, value);
    }
}
