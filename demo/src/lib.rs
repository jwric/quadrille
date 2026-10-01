//! The quadrille showcase: the console of a spacecraft that does not exist.
//!
//! Each page shows one side of the toolkit: the HUD its instruments, the kit
//! its widgets in every state, the lab its type.
mod attitude;
mod drawing;
mod hud;
mod kit;
pub mod lab;
mod scope;
mod shell;
mod telemetry;

use iced::Widget as _;
use iced::keyboard::{self, key};
use iced::time::{self, Duration, Instant};
use iced::{Subscription, Task};
use quadrille::{Element, Theme};

pub use telemetry::Telemetry;

/// The size of the window a display at 1× shows, in virtual pixels.
pub const VIEWPORT: iced::Size = iced::Size::new(640.0, 400.0);

/// Runs the showcase.
pub fn run() -> iced::Result {
    let scale = quadrille::LOGICAL_PER_VIRTUAL as f32;

    iced::application(App::new, App::update, App::view)
        .title("quadrille")
        .settings(settings())
        .theme(App::theme)
        .subscription(App::subscription)
        .window_size((VIEWPORT.width * scale, VIEWPORT.height * scale))
        .run()
}

/// The settings the showcase runs with: the toolkit's, plus the candidate
/// fonts the lab compares.
pub fn settings() -> iced::Settings {
    let mut settings = quadrille::settings();

    settings
        .fonts
        .extend(lab::FONTS.iter().map(|font| (*font).into()));
    settings
}

/// The showcase.
pub struct App {
    page: Page,
    theme: usize,
    launch: Instant,
    telemetry: Telemetry,
    video: usize,
    scope: scope::Scope,
    drawing: drawing::Drawing,
    kit: kit::Kit,
    lab: lab::Lab,
}

/// A page of the console, chosen with the soft keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// Flight instruments.
    Hud,
    /// Signal instruments: a scope, a spectrum and its waterfall.
    Scope,
    /// Technical drawing: a cutaway with callouts, and a block diagram.
    Drawing,
    /// Every widget in every state.
    Kit,
    /// The type lab.
    Lab,
}

impl Page {
    /// Every page, in soft-key order.
    pub const ALL: [Self; 5] = [Self::Hud, Self::Scope, Self::Drawing, Self::Kit, Self::Lab];

    /// The legend on the page's soft key.
    pub fn legend(self) -> &'static str {
        match self {
            Self::Hud => "HUD",
            Self::Scope => "SCOPE",
            Self::Drawing => "DRAW",
            Self::Kit => "KIT",
            Self::Lab => "TYPE",
        }
    }
}

/// A message of the showcase.
#[derive(Debug, Clone)]
pub enum Message {
    /// The clock ticked.
    Tick,
    /// A soft key chose a page.
    Show(Page),
    /// The theme key was pressed.
    NextTheme,
    /// A video input was chosen on the HUD.
    Video(usize),
    /// Something on the scope page changed.
    Scope(scope::Message),
    /// Something on the drawing page changed.
    Drawing(drawing::Message),
    /// A widget in the kit changed.
    Kit(kit::Message),
    /// Something in the lab changed.
    Lab(lab::Message),
}

impl App {
    /// The showcase at launch, on the HUD.
    pub fn new() -> Self {
        Self::at(Page::Hud, 0, 0.0)
    }

    /// The showcase on `page` in the `theme`th theme, `elapsed` seconds after
    /// launch: the same arguments draw the same frame.
    pub fn at(page: Page, theme: usize, elapsed: f32) -> Self {
        Self {
            page,
            theme: theme % Theme::ALL.len(),
            launch: Instant::now(),
            telemetry: Telemetry::at(elapsed),
            video: 1,
            scope: scope::Scope::default(),
            drawing: drawing::Drawing::default(),
            kit: kit::Kit::default(),
            lab: lab::Lab::default(),
        }
    }

    /// Handles a [`Message`].
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.telemetry = Telemetry::at(self.launch.elapsed().as_secs_f32());

                // The waterfall's time scale assumes the scope page's rate, so
                // it only takes rows while it is shown.
                if self.page == Page::Scope {
                    self.scope.tick(&self.telemetry);
                }
            }
            Message::Show(page) => self.page = page,
            Message::NextTheme => self.theme = (self.theme + 1) % Theme::ALL.len(),
            Message::Video(input) => self.video = input,
            Message::Scope(message) => self.scope.update(message),
            Message::Drawing(message) => self.drawing.update(message),
            Message::Kit(message) => self.kit.update(message),
            Message::Lab(message) => self.lab.update(message),
        }

        Task::none()
    }

    /// Draws the console.
    pub fn view(&self) -> Element<'_, Message> {
        let page = match self.page {
            Page::Hud => hud::view(&self.telemetry, self.video),
            Page::Scope => self.scope.view(&self.telemetry).map(Message::Scope).boxed(),
            Page::Drawing => self
                .drawing
                .view(&self.telemetry)
                .map(Message::Drawing)
                .boxed(),
            Page::Kit => self.kit.view().map(Message::Kit).boxed(),
            Page::Lab => self.lab.view().map(Message::Lab).boxed(),
        };

        shell::view(self.page, &self.theme(), &self.telemetry, page)
    }

    /// The theme in use.
    pub fn theme(&self) -> Theme {
        Theme::ALL[self.theme].clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        let keys = keyboard::listen().filter_map(|event| {
            let keyboard::Event::KeyPressed { key, .. } = event else {
                return None;
            };

            match key.as_ref() {
                keyboard::Key::Named(key::Named::F1) => Some(Message::Show(Page::Hud)),
                keyboard::Key::Named(key::Named::F2) => Some(Message::Show(Page::Scope)),
                keyboard::Key::Named(key::Named::F3) => Some(Message::Show(Page::Drawing)),
                keyboard::Key::Named(key::Named::F4) => Some(Message::Show(Page::Kit)),
                keyboard::Key::Named(key::Named::F5) => Some(Message::Show(Page::Lab)),
                keyboard::Key::Named(key::Named::F6) => Some(Message::NextTheme),
                _ => None,
            }
        });

        // The HUD moves at fifteen frames a second; elsewhere only the clock
        // in the status bar does, and a tenth of a second is its finest digit
        // worth watching.
        let period = match self.page {
            Page::Hud | Page::Scope | Page::Drawing => 66,
            Page::Kit | Page::Lab => 100,
        };

        // The timer's instant is a different type on the web; the tick only
        // says when, and the clock is read in `update`.
        let clock = time::every(Duration::from_millis(period)).map(|_| Message::Tick);

        Subscription::batch([keys, clock])
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// The browser entry point: the same showcase, with panics shown on the page.
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod web {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(start)]
    pub fn start() {
        std::panic::set_hook(Box::new(|info| {
            console_error_panic_hook::hook(info);

            let message = info.to_string();

            if let Some(output) = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.get_element_by_id("boot-error"))
            {
                output.set_text_content(Some(&message));
                let _ = output.set_attribute("style", "display: block");
            }
        }));

        if let Err(error) = super::run() {
            web_sys::console::error_1(&error.to_string().into());
        }
    }
}
