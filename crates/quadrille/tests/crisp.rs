//! Every pixel the toolkit draws is a colour of the palette: nothing is
//! smoothed, and nothing is blended except where a style asks for a mix.
use iced::Widget as _;
use iced::widget::column;
use iced_test::Simulator;
use quadrille::widget::{self, bar, button, checkbox, group, indicator, inverse, label, radio};
use quadrille::{Element, Palette, Theme, px};

fn panel<'a>() -> Element<'a, ()> {
    group(
        "PANEL",
        column![
            label("LABEL 0123 °C"),
            // Departure Mono has no `ƀ`. It must be drawn as the face's own
            // missing glyph, not borrowed from a smooth system font.
            label("MISSING ƀ"),
            bar(0.0..=1.0, 0.6).width(80.0),
            bar(0.0..=1.0, 0.6).segments(3).redline(0.5).width(80.0),
            indicator("LAMP", true),
            indicator("LAMP", false),
            checkbox("CHECK", true, |_| ()),
            radio("RADIO", 1, Some(1), |_| ()),
            button("PRESS").on_press(()),
            inverse(label("INVERSE")),
            widget::divider(),
        ]
        .spacing(px::GAP),
    )
    .boxed()
}

fn roles(palette: &Palette) -> [iced::Color; 15] {
    [
        palette.void,
        palette.ground,
        palette.raised,
        palette.hover,
        palette.edge,
        palette.ink,
        palette.muted,
        palette.faint,
        palette.line,
        palette.accent,
        palette.on_accent,
        palette.highlight,
        palette.live,
        palette.caution,
        palette.alarm,
    ]
}

/// The pixels of a snapshot, as RGBA rows.
fn pixels(snapshot: &iced_test::simulator::Snapshot, name: &str) -> (u32, Vec<u8>) {
    let directory = std::env::temp_dir().join(format!("quadrille-crisp-{}", std::process::id()));
    let path = directory.join(format!("{name}.png"));

    let _ = std::fs::remove_dir_all(&directory);
    assert!(snapshot.matches_image(&path).expect("write the snapshot"));

    let written = std::fs::read_dir(&directory)
        .expect("read the snapshot directory")
        .flatten()
        .find(|entry| entry.file_name().to_string_lossy().starts_with(name))
        .expect("the snapshot was written")
        .path();

    let decoder = png::Decoder::new(std::io::BufReader::new(
        std::fs::File::open(&written).expect("open the snapshot"),
    ));
    let mut reader = decoder.read_info().expect("read the snapshot header");
    let mut bytes = vec![0; reader.output_buffer_size().expect("a small snapshot")];
    let info = reader.next_frame(&mut bytes).expect("decode the snapshot");

    let _ = std::fs::remove_dir_all(&directory);
    bytes.truncate(info.buffer_size());

    (info.width, bytes)
}

#[test]
fn every_pixel_is_a_palette_colour() {
    for theme in Theme::ALL {
        let mut ui = Simulator::with_size(quadrille::settings(), (160.0, 200.0), panel());
        let snapshot = ui.snapshot(theme).expect("draw the panel");
        let (width, bytes) = pixels(&snapshot, theme.name());

        let allowed: Vec<[u8; 3]> = roles(theme.palette())
            .iter()
            .map(|color| {
                let [r, g, b, _] = color.into_rgba8();
                [r, g, b]
            })
            .collect();

        let strays: Vec<(u32, u32, [u8; 3])> = bytes
            .chunks_exact(4)
            .enumerate()
            .map(|(i, pixel)| {
                (
                    i as u32 % width,
                    i as u32 / width,
                    [pixel[0], pixel[1], pixel[2]],
                )
            })
            .filter(|(_, _, rgb)| !allowed.contains(rgb))
            .collect();

        assert!(
            strays.is_empty(),
            "{theme}: {} pixels off the palette, first {:?}",
            strays.len(),
            &strays[..strays.len().min(8)],
        );
    }
}
