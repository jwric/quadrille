use std::ops::RangeInclusive;

use iced_widget::core::{Color, Point, Rectangle};
use iced_widget::graphics::geometry;

use super::{Anchor, Lettering, Pen, rectangle};
use crate::Face;

/// A note on a drawing: a label of one or more lines, each on a shelf, tied
/// to a point by a leader.
///
/// The leader runs straight from the target to the elbow, the inner end of
/// the first shelf, and the shelves run away from the target. Each line after
/// the first sits on a shelf of its own, hung from the first by a drop.
///
/// ```text
///        FIRST LINE
///    ──┬─────────────
///   /  │  SECOND LINE
///  /   └──────────
/// ·
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note<'a> {
    /// The point the leader starts from.
    pub target: Point<i32>,
    /// Where the leader meets the first shelf.
    pub elbow: Point<i32>,
    /// The label, a line per `\n`.
    pub label: Lettering<'a>,
}

impl<'a> Note<'a> {
    /// Pixels from the elbow to the nearest column of ink.
    const LEAD: i32 = 6;
    /// Pixels from the elbow to the drop.
    const DROP: i32 = 2;
    /// Pixels a shelf reaches past the ink at its far end.
    const OVERHANG: i32 = 2;

    /// A note on `target`, its elbow on the target until it is placed.
    pub fn new(target: Point<i32>, label: Lettering<'a>) -> Self {
        Self {
            target,
            elbow: target,
            label,
        }
    }

    /// The same note with its elbow at `elbow`.
    pub fn at(self, elbow: Point<i32>) -> Self {
        Self { elbow, ..self }
    }

    /// Whether the shelves run right from the elbow: they run away from the
    /// target.
    pub fn rightwards(&self) -> bool {
        self.elbow.x >= self.target.x
    }

    /// The rectangle holding the label and its shelves.
    pub fn bounds(&self) -> Rectangle<i32> {
        let shelves = self.shelves();
        let left = shelves.iter().map(|shelf| shelf.2.min(shelf.3)).min();
        let right = shelves.iter().map(|shelf| shelf.2.max(shelf.3)).max();
        let (left, right) = (left.unwrap_or(self.elbow.x), right.unwrap_or(self.elbow.x));
        let top = self.elbow.y - self.above();

        rectangle(left, top, right - left + 1, self.above() + self.below() + 1)
    }

    /// Rows of the label above the elbow: the first line's capitals and the
    /// row between them and the shelf.
    fn above(&self) -> i32 {
        i32::from(self.label.face.cap()) + 1
    }

    /// Rows of the label below the elbow, down to the last shelf.
    fn below(&self) -> i32 {
        (self.label.text.lines().count().max(1) as i32 - 1) * pitch(self.label.face)
    }

    /// Each line with its shelf: the text's left edge, the shelf's row, and
    /// the shelf's inner and outer ends.
    fn shelves(&self) -> Vec<(i32, i32, i32, i32)> {
        let face = self.label.face;
        let (left_bearing, right_bearing) = bearings(face);
        let sign = if self.rightwards() { 1 } else { -1 };

        self.label
            .text
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let width = i32::from(face.width(line));
                let row = self.elbow.y + i as i32 * pitch(face);
                let inner = if i == 0 {
                    self.elbow.x
                } else {
                    self.elbow.x + sign * Self::DROP
                };

                let (x, outer) = if self.rightwards() {
                    let ink = self.elbow.x + Self::LEAD;
                    let x = ink - left_bearing;

                    (x, x + width - 1 - right_bearing + Self::OVERHANG)
                } else {
                    let ink = self.elbow.x - Self::LEAD;
                    let x = ink + 1 + right_bearing - width;

                    (x, x + left_bearing - Self::OVERHANG)
                };

                (x, row, inner, outer)
            })
            .collect()
    }
}

/// The rows from one shelf to the next.
fn pitch(face: Face) -> i32 {
    i32::from(face.line())
}

/// The empty columns left and right of a typical glyph in its cell.
fn bearings(face: Face) -> (i32, i32) {
    let air = face.advance();
    let padding = face.padding(air, 0, 0);

    (
        i32::from(air) - padding.left as i32,
        i32::from(air) - padding.right as i32,
    )
}

/// Lays out `notes` down one side of a drawing by setting their elbows.
///
/// Every elbow is on column `column`. Each first shelf goes as near as it
/// can to the row where a leader from its target, rising one pixel for every
/// two across, would meet the column: rising for targets above row `pivot`,
/// falling for the rest. The notes keep that order down the column, stay
/// apart, and stay within `rows` when they fit. Leaders drawn this way fan
/// out without crossing the labels.
pub fn fan(notes: &mut [Note<'_>], column: i32, rows: RangeInclusive<i32>, pivot: i32) {
    const GAP: i32 = 4;

    let ideal = |note: &Note<'_>| {
        let rise = (column - note.target.x).abs() / 2;

        if note.target.y < pivot {
            note.target.y - rise
        } else {
            note.target.y + rise
        }
    };

    for note in notes.iter_mut() {
        note.elbow = Point::new(column, note.target.y);
    }

    let mut order: Vec<usize> = (0..notes.len()).collect();
    order.sort_by_key(|&i| (ideal(&notes[i]), notes[i].target.y, notes[i].target.x));

    // Each run of notes packed a gap apart is a cluster, placed where its
    // notes want to be on average. Clusters that overlap merge.
    struct Cluster {
        start: usize,
        end: usize,
        top: i32,
        offsets: Vec<i32>,
        height: i32,
    }

    let place = |start: usize, end: usize| {
        let mut offsets = Vec::with_capacity(end - start);
        let mut offset = 0;
        let mut previous: Option<&Note<'_>> = None;

        for &i in &order[start..end] {
            let note = &notes[i];

            offset += match previous {
                Some(previous) => previous.below() + 1 + GAP + note.above(),
                None => note.above(),
            };
            offsets.push(offset);
            previous = Some(note);
        }

        let height = offset + previous.map_or(0, Note::below) + 1;
        let wanted: i64 = order[start..end]
            .iter()
            .zip(&offsets)
            .map(|(&i, offset)| i64::from(ideal(&notes[i]) - offset))
            .sum();
        let wanted = wanted.div_euclid((end - start) as i64) as i32;
        let lowest = rows.end() - height + 1;
        let top = wanted.min(lowest).max(*rows.start());

        Cluster {
            start,
            end,
            top,
            offsets,
            height,
        }
    };

    let mut clusters: Vec<Cluster> = Vec::new();

    for k in 0..order.len() {
        let mut cluster = place(k, k + 1);

        while let Some(previous) = clusters.last() {
            if previous.top + previous.height + GAP <= cluster.top {
                break;
            }

            let start = previous.start;
            clusters.pop();
            cluster = place(start, cluster.end);
        }

        clusters.push(cluster);
    }

    for cluster in clusters {
        for (&i, offset) in order[cluster.start..cluster.end]
            .iter()
            .zip(&cluster.offsets)
        {
            notes[i].elbow.y = cluster.top + offset;
        }
    }
}

impl<Renderer> Pen<'_, Renderer>
where
    Renderer: geometry::Renderer,
{
    /// A [`Note`]: its leader, shelves and drop in `line`, and its label in
    /// the label's colour.
    pub fn note(&mut self, note: &Note<'_>, line: Color) {
        if note.target != note.elbow {
            self.line(note.target, note.elbow, line);
        }

        let shelves = note.shelves();

        if let Some(&(_, last, inner, _)) = shelves.get(1..).and_then(<[_]>::last) {
            self.vline(inner, note.elbow.y + 1, last, line);
        }

        for (i, &(x, row, inner, outer)) in shelves.iter().enumerate() {
            let text = note.label.text.lines().nth(i).unwrap_or_default();

            self.hline(inner, outer, row, line);
            self.text(
                note.label.face,
                text,
                Point::new(x, row - 1),
                Anchor::BASELINE_LEFT,
                note.label.color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(target: (i32, i32), text: &str) -> Note<'_> {
        Note::new(
            Point::new(target.0, target.1),
            Lettering::new(Face::BODY, text, Color::BLACK),
        )
    }

    #[test]
    fn shelves_run_away_from_the_target() {
        let right = note((0, 40), "GYRO").at(Point::new(20, 20));
        let left = note((40, 40), "GYRO").at(Point::new(20, 20));

        assert!(right.rightwards());
        assert!(!left.rightwards());

        // Four cells of six with the ink from 26 to 48, and the shelf two
        // past it.
        assert_eq!(right.bounds(), rectangle(20, 11, 31, 10));

        // Mirrored about the elbow.
        assert_eq!(left.bounds(), rectangle(-10, 11, 31, 10));
    }

    #[test]
    fn a_second_line_hangs_a_line_below() {
        let two = note((0, 40), "HF RESCUE\nREC/TRANS").at(Point::new(20, 20));
        let shelves = two.shelves();

        assert_eq!(shelves.len(), 2);
        assert_eq!(shelves[1].1, 20 + 12);
        assert_eq!(shelves[1].2, 22);
        assert_eq!(shelves[0].0, shelves[1].0);
        assert_eq!(two.bounds().height, 9 + 12 + 1);
    }

    #[test]
    fn a_lone_note_sits_where_its_leader_rises_to() {
        let mut notes = [note((60, 50), "TANK")];

        fan(&mut notes, 100, 0..=200, 100);

        assert_eq!(notes[0].elbow, Point::new(100, 30));
    }

    #[test]
    fn a_fan_keeps_notes_apart_in_order_and_in_bounds() {
        let mut notes = [
            note((50, 40), "A"),
            note((52, 42), "B\nB"),
            note((54, 44), "C"),
            note((50, 150), "D"),
            note((48, 152), "E"),
            note((60, 10), "F"),
        ];

        fan(&mut notes, 100, 4..=190, 100);

        let mut placed: Vec<_> = notes.iter().map(Note::bounds).collect();
        placed.sort_by_key(|bounds| bounds.y);

        for pair in placed.windows(2) {
            assert!(pair[0].y + pair[0].height + 4 <= pair[1].y, "{pair:?}");
        }

        assert!(placed.iter().all(|b| b.y >= 4 && b.y + b.height - 1 <= 190));
        assert!(notes.iter().all(|note| note.elbow.x == 100));
        assert!(notes[0].elbow.y < notes[1].elbow.y);
        assert!(notes[1].elbow.y < notes[2].elbow.y);
        assert!(notes[3].elbow.y > notes[3].target.y);
    }
}
