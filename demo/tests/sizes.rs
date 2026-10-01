//! Every page draws at any window size, from a sliver to a large screen.
use std::sync::{Mutex, PoisonError};

use iced_test::Simulator;
use quadrille_demo::{App, Page, settings};

/// Simulators drawing at once on separate threads can bring the process
/// down in the graphics driver, so they take turns.
static TURN: Mutex<()> = Mutex::new(());

/// Draws `page` in a window `width` × `height` virtual pixels.
fn draw(page: Page, width: f32, height: f32) {
    let _turn = TURN.lock().unwrap_or_else(PoisonError::into_inner);
    let app = App::at(page, 0, 42.0);
    let mut ui = Simulator::with_size(settings(), (width, height), app.view());

    ui.snapshot(&app.theme())
        .unwrap_or_else(|error| panic!("{page:?} at {width} × {height}: {error}"));
}

#[test]
fn every_page_draws_from_a_sliver_to_a_large_screen() {
    for page in Page::ALL {
        for (width, height) in [(40.0, 30.0), (200.0, 150.0), (960.0, 600.0)] {
            draw(page, width, height);
        }
    }
}

/// Between these widths the bench's right column narrows until the heat
/// map's frame is too narrow for the name of the sun above it.
#[test]
fn the_bench_draws_as_its_right_column_gives_out() {
    for width in (360..=384).step_by(4) {
        draw(Page::Scope, width as f32, 400.0);
    }
}
