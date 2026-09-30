use graticule::widget::{Knob, knob};
use graticule::{Element, Theme};
use iced::{Event, Point, mouse};
use iced_test::Simulator;

#[derive(Debug, Clone, PartialEq)]
struct Turned(i32);

fn simulate(knob: Knob<'static, Turned>) -> Simulator<'static, Turned, Theme> {
    Simulator::with_settings(graticule::settings(), Element::from(knob))
}

fn wheel(delta: mouse::ScrollDelta) -> Event {
    Event::Mouse(mouse::Event::WheelScrolled { delta })
}

#[test]
fn a_wheel_notch_turns_the_knob_a_step() {
    let mut ui = simulate(knob(0..=10, 4, 2, Turned));

    ui.point_at(Point::new(7.0, 7.0));
    let _ = ui.simulate([wheel(mouse::ScrollDelta::Lines { x: 0.0, y: 3.0 })]);
    let _ = ui.simulate([wheel(mouse::ScrollDelta::Pixels { x: 0.0, y: 40.0 })]);

    assert_eq!(ui.into_messages().collect::<Vec<_>>(), vec![Turned(6)]);
}

#[test]
fn a_drag_up_its_travel_turns_the_knob_through_its_range() {
    let mut ui = simulate(knob(0..=10, 0, 1, Turned));

    ui.point_at(Point::new(7.0, 7.0));
    let _ = ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    // The drag goes on past the knob's bounds.
    let position = Point::new(7.0, 7.0 - 96.0);

    ui.point_at(position);
    let _ = ui.simulate([Event::Mouse(mouse::Event::CursorMoved { position })]);
    let _ = ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    assert_eq!(ui.into_messages().collect::<Vec<_>>(), vec![Turned(10)]);
}

#[test]
fn the_wheel_away_from_the_knob_does_nothing() {
    let mut ui = simulate(knob(0..=10, 5, 1, Turned));

    ui.point_at(Point::new(40.0, 40.0));
    let _ = ui.simulate([wheel(mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 })]);

    assert_eq!(ui.into_messages().count(), 0);
}
