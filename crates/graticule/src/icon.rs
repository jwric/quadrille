//! The toolkit's pixel icons, as [`Sprite`]s.
//!
//! Every icon has odd sides, so it has a centre pixel, and is no taller than
//! the capitals of [`Face::BODY`](crate::Face::BODY), so it sits in a line of
//! text without growing it. Show one with [`widget::icon`](crate::widget::icon)
//! or draw it with [`Pen::sprite`](crate::draw::Pen::sprite).
use crate::draw::Sprite;

/// An empty check box.
pub const BOX: Sprite = Sprite::new(&[
    "#######", "#.....#", "#.....#", "#.....#", "#.....#", "#.....#", "#######",
]);

/// A checked box: a solid square inside the frame.
pub const BOX_CHECKED: Sprite = Sprite::new(&[
    "#######", "#.....#", "#.###.#", "#.###.#", "#.###.#", "#.....#", "#######",
]);

/// An empty radio button.
pub const RADIO: Sprite = Sprite::new(&[
    "..###..", ".#...#.", "#.....#", "#.....#", "#.....#", ".#...#.", "..###..",
]);

/// A selected radio button.
pub const RADIO_SELECTED: Sprite = Sprite::new(&[
    "..###..", ".#...#.", "#.###.#", "#.###.#", "#.###.#", ".#...#.", "..###..",
]);

/// A tick.
pub const TICK: Sprite = Sprite::new(&[
    "......#", ".....##", "#...##.", "##.##..", ".###...", "..#....", ".......",
]);

/// A cross.
pub const CROSS: Sprite = Sprite::new(&[
    "#.....#", ".#...#.", "..#.#..", "...#...", "..#.#..", ".#...#.", "#.....#",
]);

/// A plus.
pub const PLUS: Sprite = Sprite::new(&[
    "...#...", "...#...", "...#...", "#######", "...#...", "...#...", "...#...",
]);

/// A minus.
pub const MINUS: Sprite = Sprite::new(&[
    ".......", ".......", ".......", "#######", ".......", ".......", ".......",
]);

/// A chevron pointing up.
pub const CHEVRON_UP: Sprite = Sprite::new(&[
    ".......", ".......", "...#...", "..###..", ".##.##.", "##...##", ".......",
]);

/// A chevron pointing down.
pub const CHEVRON_DOWN: Sprite = Sprite::new(&[
    ".......", "##...##", ".##.##.", "..###..", "...#...", ".......", ".......",
]);

/// A chevron pointing left.
pub const CHEVRON_LEFT: Sprite = Sprite::new(&[
    "....#..", "...##..", "..##...", ".##....", "..##...", "...##..", "....#..",
]);

/// A chevron pointing right.
pub const CHEVRON_RIGHT: Sprite = Sprite::new(&[
    "..#....", "..##...", "...##..", "....##.", "...##..", "..##...", "..#....",
]);

/// A diamond.
pub const DIAMOND: Sprite = Sprite::new(&[
    "...#...", "..#.#..", ".#...#.", "#.....#", ".#...#.", "..#.#..", "...#...",
]);

/// A solid diamond.
pub const DIAMOND_SOLID: Sprite = Sprite::new(&[
    "...#...", "..###..", ".#####.", "#######", ".#####.", "..###..", "...#...",
]);

/// A target: a ring around a dot.
pub const TARGET: Sprite = Sprite::new(&[
    "..###..", ".#...#.", "#.....#", "#..#..#", "#.....#", ".#...#.", "..###..",
]);

/// A warning: a triangle with a bar.
pub const WARNING: Sprite = Sprite::new(&[
    "...#...", "..#.#..", "..#.#..", ".#.#.#.", ".#...#.", "#..#..#", "#######",
]);

/// Every icon, with its name, for a specimen.
pub const ALL: [(&str, Sprite); 16] = [
    ("BOX", BOX),
    ("BOX_CHECKED", BOX_CHECKED),
    ("RADIO", RADIO),
    ("RADIO_SELECTED", RADIO_SELECTED),
    ("TICK", TICK),
    ("CROSS", CROSS),
    ("PLUS", PLUS),
    ("MINUS", MINUS),
    ("CHEVRON_UP", CHEVRON_UP),
    ("CHEVRON_DOWN", CHEVRON_DOWN),
    ("CHEVRON_LEFT", CHEVRON_LEFT),
    ("CHEVRON_RIGHT", CHEVRON_RIGHT),
    ("DIAMOND", DIAMOND),
    ("DIAMOND_SOLID", DIAMOND_SOLID),
    ("TARGET", TARGET),
    ("WARNING", WARNING),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Face;

    #[test]
    fn every_icon_has_a_centre_and_fits_the_capitals() {
        for (name, icon) in ALL {
            assert_eq!(icon.width() % 2, 1, "{name} has an even width");
            assert_eq!(icon.height() % 2, 1, "{name} has an even height");
            assert!(
                icon.height() <= i32::from(Face::BODY.cap()),
                "{name} is taller than a capital"
            );
        }
    }
}
