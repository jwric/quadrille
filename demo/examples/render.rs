//! Renders every page of the showcase in every theme to PNG, headless.
//!
//! ```sh
//! cargo run -p quadrille-demo --example render -- target/render
//! ```
//!
//! The images are virtual-pixel frames: one pixel per virtual pixel, the
//! picture the display upscales.
use iced_test::Simulator;
use quadrille::Theme;
use quadrille_demo::{App, Page, VIEWPORT, settings};

fn main() {
    let directory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/render".to_owned());
    let elapsed = std::env::args()
        .nth(2)
        .and_then(|elapsed| elapsed.parse().ok())
        .unwrap_or(42.0);

    std::fs::create_dir_all(&directory).expect("create the output directory");

    if std::env::args().any(|argument| argument == "--lab") {
        return lab(&directory, elapsed);
    }

    for page in Page::ALL {
        for (index, theme) in Theme::ALL.iter().enumerate() {
            let app = App::at(page, index, elapsed);
            let name = format!(
                "{}-{}",
                page.legend().to_lowercase(),
                theme.name().to_lowercase()
            );
            let path = format!("{directory}/{name}.png");

            // A snapshot only writes an image that is not there yet.
            for entry in std::fs::read_dir(&directory)
                .expect("read the output directory")
                .flatten()
            {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{name}-"))
                {
                    let _ = std::fs::remove_file(entry.path());
                }
            }

            let mut ui = Simulator::with_size(settings(), VIEWPORT, app.view());
            let snapshot = ui.snapshot(&app.theme()).expect("draw the page");

            snapshot.matches_image(&path).expect("write the image");
            println!("{name}");
        }
    }
}

/// Renders the type lab once per candidate, and its ladder.
fn lab(directory: &str, elapsed: f32) {
    use quadrille_demo::{Message, lab};

    let views = (0..lab::count())
        .map(|i| {
            (
                format!("lab-{i:02}"),
                vec![Message::Lab(lab::Message::Select(i))],
            )
        })
        .chain([
            (
                "lab-ladder".to_owned(),
                vec![Message::Lab(lab::Message::View(lab::View::Ladder))],
            ),
            (
                "lab-system".to_owned(),
                vec![Message::Lab(lab::Message::View(lab::View::System))],
            ),
        ]);

    for (name, messages) in views {
        let mut app = App::at(Page::Lab, 0, elapsed);

        for message in messages {
            let _ = app.update(message);
        }

        render(directory, &name, &app);
        println!("{name}");
    }
}

fn render(directory: &str, name: &str, app: &App) {
    for entry in std::fs::read_dir(directory)
        .expect("read the output directory")
        .flatten()
    {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(&format!("{name}-"))
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    let mut ui = Simulator::with_size(settings(), VIEWPORT, app.view());
    let snapshot = ui.snapshot(&app.theme()).expect("draw the page");

    snapshot
        .matches_image(format!("{directory}/{name}.png"))
        .expect("write the image");
}
