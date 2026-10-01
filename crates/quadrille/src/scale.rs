//! The arithmetic of scales: engineering notation, round steps, where labels
//! go, and which samples land in which column.
//!
//! None of it draws. A visualization asks these functions where its ticks,
//! labels and columns fall, then draws them with a [`Pen`](crate::draw::Pen),
//! so every chart, gauge and readout on a quadrille interface drops a label
//! the same way and decimates its samples the same way.
use std::ops::{Range, RangeInclusive};

const PREFIXES: [&str; 9] = ["p", "n", "µ", "m", "", "k", "M", "G", "T"];
const UNPREFIXED: i32 = 4;

/// `value` to three significant figures in engineering notation, then a
/// space and `unit` with its prefix: `500 µs`, `1.00 km`, `-40.0 mV`.
pub fn engineering(value: f32, unit: &str) -> String {
    let (number, prefix) = split(value, 3);

    format!("{number} {prefix}{unit}")
}

/// `value` as a number and an SI prefix, to `figures` significant figures.
fn split(value: f32, figures: i32) -> (String, &'static str) {
    if !value.is_finite() {
        return ("--".to_owned(), "");
    }

    let value = if value == 0.0 { 0.0 } else { value };
    let magnitude = value.abs();
    let mut power = if magnitude == 0.0 {
        0
    } else {
        (magnitude.log10() / 3.0).floor() as i32
    }
    .clamp(-UNPREFIXED, UNPREFIXED);

    let decimals = |x: f32| {
        let digits = if x == 0.0 {
            1
        } else {
            (x.abs().log10().floor() as i32 + 1).max(1)
        };

        (figures - digits).max(0)
    };
    let round = |x: f32, decimals: i32| {
        let shift = 10_f32.powi(decimals);

        (x * shift).round() / shift
    };

    loop {
        let scaled = value / 1000_f32.powi(power);
        let rounded = round(scaled, decimals(scaled));

        // Rounding can carry into the next prefix: 999.7 is 1.00 k.
        if rounded.abs() >= 1000.0 && power < UNPREFIXED {
            power += 1;
            continue;
        }

        // And into the next figure: 9.996 is 10.0.
        let decimals = decimals(rounded) as usize;

        return (
            format!("{scaled:.decimals$}"),
            PREFIXES[(power + UNPREFIXED) as usize],
        );
    }
}

/// The label of a tick at `value` on a scale stepping by `step`: as few
/// figures as the step needs, and an SI prefix chosen by `magnitude`, the
/// largest value on the scale. `5k`, `2.5k`, `0`.
pub fn tick(value: f32, step: f32, magnitude: f32) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }

    let power = if magnitude == 0.0 {
        0
    } else {
        (magnitude.abs().log10() / 3.0).floor() as i32
    }
    .clamp(-UNPREFIXED, UNPREFIXED);

    let scale = 1000_f32.powi(power);
    let unit = step.abs() / scale;
    let decimals = (0..3)
        .find(|&d| {
            let shifted = unit * 10_f32.powi(d);

            (shifted - shifted.round()).abs() < 1e-3
        })
        .unwrap_or(3) as usize;

    format!(
        "{:.decimals$}{}",
        value / scale,
        PREFIXES[(power + UNPREFIXED) as usize]
    )
}

/// The step of 1, 2 or 5 times a power of ten that divides `span` into at
/// most `count` intervals.
pub fn step(span: f32, count: u32) -> f32 {
    let raw = span.abs() / count.max(1) as f32;

    if raw == 0.0 || !raw.is_finite() {
        return 1.0;
    }

    let power = 10_f32.powf(raw.log10().floor());

    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|mantissa| mantissa * power)
        .find(|step| *step >= raw * 0.999)
        .unwrap_or(10.0 * power)
}

/// The left edge of each label along a scale, or `None` for a label left
/// off.
///
/// `labels` are `(anchor, width)` pairs in order along the scale. Each label
/// is centred on its anchor, then moved inside `span` if it would cross an
/// end. The last label is kept first; any other is kept only where it clears
/// the labels kept either side of it by `gap` pixels. A label is never cut:
/// one that cannot fit is dropped, and its tick stands without it.
pub fn place(labels: &[(i32, i32)], span: RangeInclusive<i32>, gap: i32) -> Vec<Option<i32>> {
    let (start, end) = (*span.start(), *span.end());

    let lefts: Vec<Option<i32>> = labels
        .iter()
        .map(|&(anchor, width)| {
            (width <= end - start + 1)
                .then(|| (anchor - width.div_euclid(2)).clamp(start, end + 1 - width))
        })
        .collect();

    let mut placed = vec![None; labels.len()];
    let Some(last) = lefts.iter().rposition(Option::is_some) else {
        return placed;
    };

    placed[last] = lefts[last];

    let limit = lefts[last].unwrap_or(end + 1) - gap;
    let mut reach = start - gap;

    for i in 0..last {
        let Some(left) = lefts[i] else {
            continue;
        };
        let right = left + labels[i].1;

        if left >= reach + gap && right <= limit {
            placed[i] = Some(left);
            reach = right;
        }
    }

    placed
}

/// Which end of a row a label is set from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// Set after the labels before it, from the left.
    Start,
    /// Set before the labels before it, from the right.
    End,
}

/// The left edge of each label in a row of `span`, or `None` for a label
/// left off.
///
/// Labels are `(width, end)` pairs in order of importance. Each is set next
/// to the last label set from its end, `gap` pixels clear of it, and is
/// dropped if it would cross a label already set or the far end of the row.
pub fn row(labels: &[(i32, End)], span: RangeInclusive<i32>, gap: i32) -> Vec<Option<i32>> {
    let (start, end) = (*span.start(), *span.end() + 1);
    let mut taken: Vec<Range<i32>> = Vec::new();
    let (mut left, mut right) = (start, end);

    labels
        .iter()
        .map(|&(width, from)| {
            let x = match from {
                End::Start => left,
                End::End => right - width,
            };
            let span = x..x + width;
            let clear =
                |other: &Range<i32>| span.end + gap <= other.start || other.end + gap <= span.start;

            if width <= 0 || x < start || span.end > end || !taken.iter().all(clear) {
                return None;
            }

            match from {
                End::Start => left = span.end + gap,
                End::End => right = x - gap,
            }

            taken.push(span);

            Some(x)
        })
        .collect()
}

/// The bins shown in `column` when `bins` bins are spread across `columns`
/// columns.
///
/// A bin belongs to the column its middle falls in, so a value at the middle
/// of a bin and the bin itself land in the same column. Every column
/// shows at least one bin: with more bins than columns the ranges tile
/// without overlap, and with fewer a bin is repeated across the columns it
/// covers.
pub fn bins(column: usize, columns: usize, bins: usize) -> Range<usize> {
    if columns == 0 || bins == 0 {
        return 0..0;
    }

    let edge = |column: usize| (2 * column * bins + columns) / (2 * columns);
    let start = edge(column).min(bins - 1);

    start..edge(column + 1).clamp(start + 1, bins)
}

/// The lowest and the highest sample in each of `columns` columns when
/// `samples` are spread evenly across them, the first at the left edge and
/// the last at the right.
///
/// With more samples than columns every sample lands in a column, so a
/// spike one sample wide still reaches its peak. With fewer, a column
/// between two samples takes the value on the line joining them.
pub fn extremes(samples: &[f32], columns: usize) -> Vec<(f32, f32)> {
    let count = samples.len();

    if count == 0 || columns == 0 {
        return Vec::new();
    }

    if count >= columns {
        return (0..columns)
            .map(|column| {
                samples[bins(column, columns, count)]
                    .iter()
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), &v| {
                        (low.min(v), high.max(v))
                    })
            })
            .collect();
    }

    (0..columns)
        .map(|column| {
            let position = if columns == 1 {
                0.0
            } else {
                column as f32 * (count - 1) as f32 / (columns - 1) as f32
            };
            let before = (position.floor() as usize).min(count - 1);
            let after = (before + 1).min(count - 1);
            let t = position - before as f32;
            let value = samples[before] + (samples[after] - samples[before]) * t;

            (value, value)
        })
        .collect()
}

/// Stretches each column's run of rows, `(top, bottom)`, to meet the runs
/// beside it, so a trace drawn as one run per column is unbroken.
///
/// The rows between two runs that do not touch are split between them, the
/// upper half to the higher run: a step reads as a stair, not a smear.
pub fn bridge(runs: &mut [(i32, i32)]) {
    if runs.len() < 2 {
        return;
    }

    let original = runs.to_vec();

    for i in 1..original.len() {
        let (before, after) = (original[i - 1], original[i]);

        if after.0 > before.1 + 1 {
            let gap = after.0 - before.1 - 1;
            let split = before.1 + gap / 2;

            runs[i - 1].1 = runs[i - 1].1.max(split);
            runs[i].0 = runs[i].0.min(split + 1);
        } else if after.1 < before.0 - 1 {
            let gap = before.0 - after.1 - 1;
            let split = before.0 - gap / 2;

            runs[i - 1].0 = runs[i - 1].0.min(split);
            runs[i].1 = runs[i].1.max(split - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engineering_keeps_three_figures() {
        assert_eq!(engineering(0.0005, "s"), "500 µs");
        assert_eq!(engineering(1000.0, "m"), "1.00 km");
        assert_eq!(engineering(-0.04, "V"), "-40.0 mV");
        assert_eq!(engineering(7_020_000.0, "B"), "7.02 MB");
        assert_eq!(engineering(0.0, "V"), "0.00 V");
        assert_eq!(engineering(2.5, "V"), "2.50 V");
    }

    #[test]
    fn engineering_carries_rounding_into_the_next_prefix() {
        assert_eq!(engineering(999.7, "m"), "1.00 km");
        assert_eq!(engineering(9.996, "V"), "10.0 V");
        assert_eq!(engineering(99.96, "V"), "100 V");
    }

    #[test]
    fn ticks_carry_only_the_figures_their_step_needs() {
        assert_eq!(tick(5000.0, 5000.0, 20000.0), "5k");
        assert_eq!(tick(2500.0, 2500.0, 20000.0), "2.5k");
        assert_eq!(tick(0.0, 5000.0, 20000.0), "0");
        assert_eq!(tick(-40.0, 20.0, 100.0), "-40");
    }

    #[test]
    fn steps_are_round() {
        assert_eq!(step(20_000.0, 4), 5000.0);
        assert_eq!(step(20_000.0, 10), 2000.0);
        assert_eq!(step(100.0, 5), 20.0);
        assert_eq!(step(7.0, 3), 5.0);
    }

    /// Renders placed labels as art: each label is its index repeated across
    /// its width.
    fn art(labels: &[(i32, i32)], placed: &[Option<i32>], length: i32) -> String {
        let mut row = vec!['.'; length as usize];

        for (i, left) in placed.iter().enumerate() {
            if let Some(left) = left {
                for x in *left..*left + labels[i].1 {
                    row[x as usize] = char::from_digit(i as u32, 10).unwrap();
                }
            }
        }

        row.into_iter().collect()
    }

    #[test]
    fn colliding_labels_are_dropped_and_the_ends_kept() {
        let labels = [(0, 8), (10, 8), (20, 8), (30, 8), (40, 8)];
        let placed = place(&labels, 0..=40, 2);

        assert_eq!(
            art(&labels, &placed, 41),
            "00000000........22222222.........44444444"
        );
    }

    #[test]
    fn labels_that_fit_are_all_kept() {
        let labels = [(1, 3), (10, 3), (19, 3)];
        let placed = place(&labels, 0..=20, 2);

        assert_eq!(art(&labels, &placed, 21), "000......111......222");
    }

    #[test]
    fn a_label_wider_than_the_scale_is_dropped() {
        assert_eq!(place(&[(5, 30)], 0..=20, 2), vec![None]);
    }

    #[test]
    fn a_row_sets_labels_from_both_ends_and_drops_what_collides() {
        let labels = [
            (6, End::Start),
            (5, End::End),
            (6, End::Start),
            (9, End::Start),
        ];
        let placed = row(&labels, 0..=24, 1);

        assert_eq!(
            art(
                &labels.iter().map(|&(w, _)| (0, w)).collect::<Vec<_>>(),
                &placed,
                25
            ),
            "000000.222222.......11111"
        );
    }

    #[test]
    fn a_row_keeps_what_fits_between_its_ends() {
        let placed = row(
            &[(12, End::Start), (10, End::End), (4, End::End)],
            0..=20,
            1,
        );

        assert_eq!(placed, vec![Some(0), None, Some(17)]);
    }

    #[test]
    fn bins_tile_the_columns() {
        assert_eq!(bins(0, 4, 10), 0..3);
        assert_eq!(bins(3, 4, 10), 8..10);
        assert_eq!(bins(0, 10, 4), 0..1);
        assert_eq!(bins(9, 10, 4), 3..4);

        let covered: usize = (0..7).map(|c| bins(c, 7, 100).len()).sum();

        assert_eq!(covered, 100);
    }

    #[test]
    fn a_bin_lands_in_the_column_of_its_middle() {
        let (columns, count) = (314, 512);

        for bin in 0..count {
            let middle = (bin as f32 + 0.5) / count as f32 * columns as f32;
            let column = middle as usize;

            assert!(bins(column, columns, count).contains(&bin), "{bin}");
        }
    }

    #[test]
    fn decimation_keeps_a_one_sample_spike() {
        let mut samples = vec![0.0; 1000];
        samples[503] = 1.0;

        let columns = extremes(&samples, 100);

        assert_eq!(columns[50], (0.0, 1.0));
        assert!(columns.iter().filter(|(_, high)| *high > 0.0).count() == 1);
    }

    #[test]
    fn sparse_samples_are_joined_by_lines() {
        let columns = extremes(&[0.0, 4.0], 5);

        assert_eq!(
            columns.iter().map(|(low, _)| *low).collect::<Vec<_>>(),
            vec![0.0, 1.0, 2.0, 3.0, 4.0]
        );
    }

    /// Renders column runs as art: `#` for a lit pixel.
    fn runs_art(runs: &[(i32, i32)]) -> String {
        let top = runs.iter().map(|r| r.0).min().unwrap();
        let bottom = runs.iter().map(|r| r.1).max().unwrap();

        (top..=bottom)
            .map(|y| {
                runs.iter()
                    .map(|&(t, b)| if (t..=b).contains(&y) { '#' } else { '.' })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_step_is_bridged_as_a_stair() {
        let mut falling = [(0, 0), (0, 0), (0, 0), (4, 4), (4, 4), (4, 4)];
        bridge(&mut falling);

        assert_eq!(runs_art(&falling), "###...\n..#...\n...#..\n...#..\n...###");

        let mut rising = [(4, 4), (4, 4), (4, 4), (0, 0), (0, 0), (0, 0)];
        bridge(&mut rising);

        assert_eq!(runs_art(&rising), "...###\n...#..\n...#..\n..#...\n###...");
    }

    #[test]
    fn touching_runs_are_left_alone() {
        let mut runs = [(0, 2), (3, 3), (2, 5)];
        bridge(&mut runs);

        assert_eq!(runs, [(0, 2), (3, 3), (2, 5)]);
    }
}
