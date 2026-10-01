use iced_widget::core::{Color, Point, Rectangle};
use iced_widget::graphics::geometry;

use super::{Anchor, Horizontal, Pen, Vertical, rectangle};
use crate::Face;

/// The border of a drawing sheet: a band between a trim line and a border
/// line, divided into zones numbered across from the left and lettered down
/// from the top, so that a place on the sheet has a name like `B3`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sheet {
    /// The sheet, trim line included.
    pub bounds: Rectangle<i32>,
    /// Zones across.
    pub columns: u8,
    /// Zones down.
    pub rows: u8,
    /// The face of the zone marks.
    pub face: Face,
}

impl Sheet {
    /// A sheet filling `bounds`, with `columns` zones across and `rows`
    /// down, marked in [`Face::BODY`].
    pub fn new(bounds: Rectangle<i32>, columns: u8, rows: u8) -> Self {
        Self {
            bounds,
            columns: columns.max(1),
            rows: rows.clamp(1, 26),
            face: Face::BODY,
        }
    }

    /// The pixels from the trim line to the border line, both included.
    pub fn band(&self) -> i32 {
        i32::from(self.face.cap()) + 4
    }

    /// The drawing area, inside the border line.
    pub fn inner(&self) -> Rectangle<i32> {
        inset(self.bounds, self.band())
    }

    /// The zone holding `point`: its letter and its number.
    pub fn zone(&self, point: Point<i32>) -> Option<(char, u8)> {
        let border = self.border();
        let column = find(
            &self.divisions(border.x, border.width, self.columns),
            point.x,
        )?;
        let row = find(&self.divisions(border.y, border.height, self.rows), point.y)?;

        Some((char::from(b'A' + row as u8), column as u8 + 1))
    }

    /// The border line's rectangle.
    fn border(&self) -> Rectangle<i32> {
        inset(self.bounds, self.band() - 1)
    }

    /// The first pixel of each zone along one side, and one past the last.
    fn divisions(&self, start: i32, length: i32, zones: u8) -> Vec<i32> {
        (0..=i32::from(zones))
            .map(|k| start + length * k / i32::from(zones))
            .collect()
    }
}

/// `bounds` shrunk by `by` on every side.
fn inset(bounds: Rectangle<i32>, by: i32) -> Rectangle<i32> {
    rectangle(
        bounds.x + by,
        bounds.y + by,
        bounds.width - 2 * by,
        bounds.height - 2 * by,
    )
}

/// The index of the division holding `at`.
fn find(divisions: &[i32], at: i32) -> Option<usize> {
    divisions
        .windows(2)
        .position(|pair| (pair[0]..pair[1]).contains(&at))
}

/// A cell of a [`Table`]: a name over a value, like the fields of a title
/// block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field<'a> {
    /// What the value is.
    pub name: &'a str,
    /// The value.
    pub value: &'a str,
    /// The field's share of its row, against the spans of the others.
    pub span: u16,
}

impl<'a> Field<'a> {
    /// A field with a span of one.
    pub const fn new(name: &'a str, value: &'a str) -> Self {
        Self {
            name,
            value,
            span: 1,
        }
    }

    /// The same field with a share of `span`.
    pub const fn span(self, span: u16) -> Self {
        Self { span, ..self }
    }
}

/// A boxed table of [`Field`]s in rows, like a title block.
///
/// Each row is divided between its fields by their spans; each field is its
/// name over its value. A name or value too wide for its cell is left out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Table<'a> {
    /// The fields, row by row.
    pub rows: &'a [&'a [Field<'a>]],
    /// The face of names and values.
    pub face: Face,
}

impl<'a> Table<'a> {
    /// A table of `rows` in [`Face::BODY`].
    pub fn new(rows: &'a [&'a [Field<'a>]]) -> Self {
        Self {
            rows,
            face: Face::BODY,
        }
    }

    /// The pixels from one row's rule to the next: the name's capitals and
    /// the value's, with two pixels of air between, above and below.
    fn row_height(&self) -> i32 {
        2 * i32::from(self.face.cap()) + 7
    }

    /// The height of the table, outline included.
    pub fn height(&self) -> i32 {
        self.rows.len() as i32 * self.row_height() + 1
    }

    /// The columns of the rules of a row `width` pixels wide starting at `x`,
    /// outline included.
    fn rules(row: &[Field<'_>], x: i32, width: i32) -> Vec<i32> {
        let total: i32 = row.iter().map(|field| i32::from(field.span)).sum();
        let mut rules = vec![x];
        let mut spanned = 0;

        for field in row {
            spanned += i32::from(field.span);
            rules.push(x + (width - 1) * spanned / total.max(1));
        }

        rules
    }
}

impl<Renderer> Pen<'_, Renderer>
where
    Renderer: geometry::Renderer,
{
    /// The border of `sheet`: trim and border lines, zone divisions and
    /// centring marks in `line`, zone numbers and letters in `marks`.
    ///
    /// A centring mark crosses the middle of each side and reaches a few
    /// pixels into the drawing.
    pub fn sheet(&mut self, sheet: &Sheet, line: Color, marks: Color) {
        let (bounds, border) = (sheet.bounds, sheet.border());
        let face = sheet.face;
        let (right, bottom) = (bounds.x + bounds.width - 1, bounds.y + bounds.height - 1);
        let (border_right, border_bottom) =
            (border.x + border.width - 1, border.y + border.height - 1);

        self.outline(bounds, line);
        self.outline(border, line);

        let across = sheet.divisions(border.x, border.width, sheet.columns);

        for (k, pair) in across.windows(2).enumerate() {
            if k > 0 {
                self.vline(pair[0], bounds.y + 1, border.y - 1, line);
                self.vline(pair[0], border_bottom + 1, bottom - 1, line);
            }

            let number = (k + 1).to_string();
            let middle = pair[0] + (pair[1] - pair[0]).div_euclid(2);
            let anchor = Anchor::new(Horizontal::Centre, Vertical::CapTop);

            for top in [bounds.y + 2, border_bottom + 2] {
                self.text(face, number.clone(), Point::new(middle, top), anchor, marks);
            }
        }

        let down = sheet.divisions(border.y, border.height, sheet.rows);

        for (k, pair) in down.windows(2).enumerate() {
            if k > 0 {
                self.hline(bounds.x + 1, border.x - 1, pair[0], line);
                self.hline(border_right + 1, right - 1, pair[0], line);
            }

            let letter = char::from(b'A' + k as u8).to_string();
            let middle = pair[0] + (pair[1] - pair[0]).div_euclid(2);
            let band = sheet.band() - 2;

            for left in [bounds.x + 1, border_right + 1] {
                let centre = left + band.div_euclid(2);

                self.text(
                    face,
                    letter.clone(),
                    Point::new(centre, middle),
                    Anchor::CENTRE,
                    marks,
                );
            }
        }

        const REACH: i32 = 4;

        let (middle_x, middle_y) = (border.x + border.width / 2, border.y + border.height / 2);

        self.vline(middle_x, bounds.y + 1, border.y + REACH, line);
        self.vline(middle_x, border_bottom - REACH, bottom - 1, line);
        self.hline(bounds.x + 1, border.x + REACH, middle_y, line);
        self.hline(border_right - REACH, right - 1, middle_y, line);
    }

    /// `table` with its top-left corner at `at`, `width` pixels wide: rules
    /// in `line`, names in `name` and values in `value`.
    pub fn table(
        &mut self,
        table: &Table<'_>,
        at: Point<i32>,
        width: i32,
        line: Color,
        name: Color,
        value: Color,
    ) {
        let face = table.face;
        let row_height = table.row_height();
        let cap_top = i32::from(face.cap_top());
        let below = i32::from(face.cap()) + 2;

        self.outline(rectangle(at.x, at.y, width, table.height()), line);

        for (r, row) in table.rows.iter().enumerate() {
            let top = at.y + r as i32 * row_height;
            let rules = Table::rules(row, at.x, width);

            if r > 0 {
                self.hline(at.x, at.x + width - 1, top, line);
            }

            for (i, field) in row.iter().enumerate() {
                let (left, right) = (rules[i], rules[i + 1]);

                if i > 0 {
                    self.vline(left, top + 1, top + row_height - 1, line);
                }

                let room = right - left - 3;
                let x = left + 2;

                for (text, y, color) in [
                    (field.name, top + 3 - cap_top, name),
                    (field.value, top + 3 + below - cap_top, value),
                ] {
                    if i32::from(face.width(text)) <= room {
                        self.text(face, text, Point::new(x, y), Anchor::TOP_LEFT, color);
                    }
                }
            }
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zones_are_lettered_down_and_numbered_across() {
        let sheet = Sheet::new(rectangle(0, 0, 200, 100), 4, 2);
        let inner = sheet.inner();

        assert_eq!(sheet.band(), 12);
        assert_eq!(inner, rectangle(12, 12, 176, 76));
        assert_eq!(sheet.zone(Point::new(12, 12)), Some(('A', 1)));
        assert_eq!(sheet.zone(Point::new(187, 87)), Some(('B', 4)));
        assert_eq!(sheet.zone(Point::new(100, 30)), Some(('A', 3)));
        assert_eq!(sheet.zone(Point::new(5, 30)), None);
    }

    #[test]
    fn a_row_divides_by_span() {
        let row = [
            Field::new("DWG NO", "GR-0413").span(2),
            Field::new("REV", "C"),
            Field::new("SHEET", "3/7"),
        ];

        assert_eq!(Table::rules(&row, 10, 81), [10, 50, 70, 90]);
    }

    #[test]
    fn a_table_is_as_tall_as_its_rows() {
        let rows: &[&[Field<'_>]] = &[&[Field::new("TITLE", "BEACON")], &[]];

        assert_eq!(Table::new(rows).row_height(), 23);
        assert_eq!(Table::new(rows).height(), 47);
    }
}
