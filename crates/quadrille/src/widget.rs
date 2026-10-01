//! Widgets: controls and indicators, and iced's built-ins set up for the
//! pixel grid.
//!
//! The functions here are the toolkit's vocabulary. Each returns a widget set
//! in [`Face::BODY`], sized in whole pixels and styled from the [`Theme`], so
//! a panel built from them lines up without per-call tuning.
pub mod bar;
pub mod group;
pub mod icon;
pub mod knob;
pub mod lamp;

pub use bar::Bar;
pub use group::Group;
pub use icon::Icon;
pub use knob::Knob;
pub use lamp::Lamp;

use std::borrow::Borrow;

use iced_widget::core::{self, Alignment, Element, Length};
use iced_widget::text::{IntoFragment, LineHeight, Shaping};
use iced_widget::{
    Button, Column, Container, PickList, Row, Rule, Scrollable, Text, TextInput, Toggler, column,
    container, row, rule, scrollable,
};

use crate::draw::Sprite;
use crate::{Face, Theme, px, style};

/// A line of text in [`Face::BODY`].
pub fn label<'a>(content: impl IntoFragment<'a>) -> Text<'a, Theme> {
    Face::BODY.text(content)
}

/// A button: a legend on a raised face, a line of [`Face::BODY`] with two
/// pixels of air above and below.
pub fn button<'a, Message>(
    legend: impl IntoFragment<'a>,
) -> Button<'a, Message, Text<'a, Theme>, Theme>
where
    Message: 'a,
{
    iced_widget::button(Face::BODY.text(legend)).padding(Face::BODY.padding(4, 2, 2))
}

/// A tab: a button whose legend is its only mark until it is the one in
/// use, when it is lit.
pub fn tab<'a, Message>(
    legend: impl IntoFragment<'a>,
    active: bool,
) -> Button<'a, Message, Text<'a, Theme>, Theme>
where
    Message: 'a,
{
    button(legend).style(style::button::tab(active))
}

/// A segmented control: its segments in one bezel, the selected one lit.
///
/// Every segment stays pressable, including the selected one.
pub fn segmented<'a, T, Message>(
    options: impl IntoIterator<Item = (T, &'a str)>,
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> Container<'a, Row<Button<'a, Message, Text<'a, Theme>, Theme>>, Theme>
where
    T: PartialEq + Copy + 'a,
    Message: Clone + 'a,
{
    let segments = options.into_iter().map(|(value, legend)| {
        iced_widget::button(Face::BODY.text(legend))
            .padding(Face::BODY.padding(4, 1, 1))
            .style(style::button::segment(selected == Some(value)))
            .on_press(on_select(value))
    });

    container(Row::with_children(segments).spacing(px::HAIR))
        .padding(px::HAIR)
        .style(style::container::outline)
}

/// A pixel [`Icon`] of `sprite`, in the colour it inherits.
pub fn icon<Theme>(sprite: Sprite) -> Icon<Theme> {
    Icon::new(sprite)
}

/// The content of a [`checkbox`] or a [`radio`]: its mark and its legend.
pub type Choice<'a, Message, Renderer = iced_widget::Renderer> =
    Row<Element<'a, Message, Theme, Renderer>>;

/// A check box and its legend, as one pressable row.
pub fn checkbox<'a, Message, Renderer>(
    legend: impl IntoFragment<'a>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message,
) -> Button<'a, Message, Choice<'a, Message, Renderer>, Theme>
where
    Message: Clone + 'a,
    Renderer: core::text::Renderer + 'a,
{
    let mark = if checked {
        icon(crate::icon::BOX_CHECKED).color(|theme: &Theme| theme.palette().accent)
    } else {
        icon(crate::icon::BOX)
    };

    choice(mark, legend).on_press(on_toggle(!checked))
}

/// A radio button and its legend, as one pressable row.
pub fn radio<'a, T, Message, Renderer>(
    legend: impl IntoFragment<'a>,
    value: T,
    selected: Option<T>,
    on_click: impl FnOnce(T) -> Message,
) -> Button<'a, Message, Choice<'a, Message, Renderer>, Theme>
where
    T: PartialEq + Copy,
    Message: Clone + 'a,
    Renderer: core::text::Renderer + 'a,
{
    let mark = if selected == Some(value) {
        icon(crate::icon::RADIO_SELECTED).color(|theme: &Theme| theme.palette().accent)
    } else {
        icon(crate::icon::RADIO)
    };

    choice(mark, legend).on_press(on_click(value))
}

fn choice<'a, Message, Renderer>(
    mark: Icon<Theme>,
    legend: impl IntoFragment<'a>,
) -> Button<'a, Message, Choice<'a, Message, Renderer>, Theme>
where
    Message: 'a,
    Renderer: core::text::Renderer + 'a,
{
    iced_widget::button(
        row![mark.height(Face::BODY.line()), Face::BODY.text(legend)]
            .spacing(f32::from(Face::BODY.advance()))
            .align_y(Alignment::Center),
    )
    .padding(0)
    .style(style::button::ghost)
}

/// A toggler and its legend, sized to a line of [`Face::BODY`].
pub fn toggler<'a, Message>(legend: impl IntoFragment<'a>, on: bool) -> Toggler<'a, Message, Theme>
where
    Message: 'a,
{
    let face = Face::BODY;

    Toggler::new(on)
        .label(legend)
        .size(f32::from(face.cap() + 2))
        .font(face.font)
        .text_size(f32::from(face.size()))
        .line_height(line_height(face))
        .spacing(f32::from(face.advance()))
}

/// A text field in [`Face::BODY`], one line tall inside a hairline.
pub fn text_input<'a, Message>(
    placeholder: &'a str,
    value: &'a str,
) -> TextInput<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    let face = Face::BODY;

    TextInput::new(placeholder, value)
        .font(face.font)
        .size(f32::from(face.size()))
        .line_height(line_height(face))
        .padding(face.padding(3, 2, 2))
}

/// A pick list in [`Face::BODY`], as tall as a [`button`], with an arrow from
/// the face itself.
pub fn pick_list<'a, T, L, V, Message>(
    options: L,
    selected: Option<V>,
    on_select: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, L, V, Message, Theme>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone + 'a,
{
    let face = Face::BODY;

    PickList::new(selected, options, T::to_string)
        .on_select(on_select)
        .font(face.font)
        .text_size(f32::from(face.size()))
        .line_height(line_height(face))
        .padding(face.padding(4, 2, 2))
        .handle(iced_widget::pick_list::Handle::Static(
            iced_widget::pick_list::Icon {
                font: face.font,
                code_point: '▼',
                size: Some(f32::from(face.size()).into()),
                line_height: Some(line_height(face)),
                shaping: Shaping::Basic,
            },
        ))
}

/// A scrollable with a 4 px bar that keeps its distance from the content.
pub fn scroll<'a, Message, W>(content: W) -> Scrollable<'a, Message, W, Theme> {
    scrollable(content)
        .scrollbar_width(px::GAP)
        .scroller_width(px::GAP)
        .spacing(px::GAP)
}

/// A table on the pixel grid: a cell of padding either side of each column,
/// a pixel above and below each row, and hairline separators.
///
/// Columns come from [`iced_widget::table::column`].
pub fn table<'a, 'b, T, Message, Renderer>(
    columns: impl IntoIterator<Item = iced_widget::table::Column<'a, 'b, T, Message, Theme, Renderer>>,
    rows: impl IntoIterator<Item = T>,
) -> iced_widget::table::Table<'a, Message, Theme, Renderer>
where
    T: Clone,
    Renderer: core::Renderer,
{
    iced_widget::table(columns, rows)
        .padding_x(f32::from(Face::BODY.advance()))
        .padding_y(px::HAIR)
        .separator(px::HAIR)
}

/// A hairline divider across the width available.
pub fn divider<'a>() -> Rule<'a, Theme> {
    rule::horizontal(px::HAIR)
}

/// A hairline divider down the height available.
pub fn separator<'a>() -> Rule<'a, Theme> {
    rule::vertical(px::HAIR)
}

/// A value under its label, like the fields of a boarding pass.
pub fn field<'a, Message, Renderer>(
    name: impl IntoFragment<'a>,
    value: impl core::Widget<Message, Theme, Renderer> + 'a,
) -> Column<Element<'a, Message, Theme, Renderer>>
where
    Message: 'a,
    Renderer: core::text::Renderer + 'a,
{
    column![Face::BODY.text(name).style(style::text::muted), value]
}

/// A label and a value on one line, the value in ink and the label muted.
pub fn reading<'a>(
    name: impl IntoFragment<'a>,
    value: impl IntoFragment<'a>,
) -> Row<Text<'a, Theme>> {
    Row::with_children([
        Face::BODY.text(name).style(style::text::muted),
        Face::BODY.text(value),
    ])
    .spacing(f32::from(Face::BODY.advance()))
}

/// `content` knocked out of an accent block: the toolkit's emphasis, used
/// where another interface would reach for a bold weight.
pub fn inverse<'a, W>(content: W) -> Container<'a, W, Theme> {
    container(content)
        .padding(Face::BODY.padding(2, 0, 0))
        .style(style::container::inverse)
}

/// A lamp with its legend beside it.
pub fn indicator<'a, Message, Renderer>(
    legend: impl IntoFragment<'a>,
    on: bool,
) -> Row<Element<'a, Message, Theme, Renderer>>
where
    Message: 'a,
    Renderer: core::text::Renderer + 'a,
{
    let legend = Face::BODY.text(legend).style(if on {
        style::text::ink
    } else {
        style::text::muted
    });

    row![lamp::lamp(on), legend]
        .spacing(px::GAP)
        .align_y(Alignment::Center)
}

/// A group of `content` under a rule broken by its `name`.
pub fn group<'a, W>(name: impl Into<String>, content: W) -> Group<'a, W, Theme> {
    Group::new(name, content)
}

/// A knob turning `value` through `range` in steps of `step`.
pub fn knob<'a, Message>(
    range: std::ops::RangeInclusive<i32>,
    value: i32,
    step: i32,
    on_change: impl Fn(i32) -> Message + 'a,
) -> Knob<'a, Message, Theme> {
    Knob::new(range, value, step, on_change)
}

/// A bar gauge showing `value` in `range`.
pub fn bar(range: std::ops::RangeInclusive<f32>, value: f32) -> Bar<Theme> {
    Bar::new(range, value)
}

/// A panel's face that fills its space: the ground under `content`.
pub fn panel<'a, W>(content: W) -> Container<'a, W, Theme> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::container::ground)
}

fn line_height(face: Face) -> LineHeight {
    LineHeight::Absolute(f32::from(face.line()).into())
}
