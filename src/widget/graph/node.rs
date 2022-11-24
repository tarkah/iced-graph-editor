use iced::{Background, Color, Point, Rectangle, Vector};
use iced_native::widget::{tree, Tree};
use iced_native::{event, layout, mouse, renderer, Element, Layout, Shell};

use super::Event;

#[derive(Debug)]
pub enum State {
    Idle,
    Hovered,
    Translating { started_at: Point, offset: Vector },
}

impl State {
    pub(super) fn adjusted_bounds(&self, bounds: Rectangle) -> Rectangle {
        match self {
            State::Idle | State::Hovered => bounds,
            State::Translating { offset, .. } => bounds + *offset,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

pub struct Node<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: Element<'a, Message, Renderer>,
    offset: Vector,
    pub(super) edges: Vec<usize>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Node<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        offset: Vector,
        edges: Vec<usize>,
    ) -> Self {
        Self {
            content: content.into(),
            offset,
            edges,
            style: Default::default(),
        }
    }

    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Node<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub(super) fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    pub(super) fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    pub(super) fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    pub(super) fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    pub(super) fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let padding = [20, 5, 5, 5].into();

        let content = self
            .content
            .as_widget()
            .layout(renderer, &limits.pad(padding));

        let node = content.size().pad(padding);

        let offset = Vector::new(padding.left as f32, padding.top as f32);

        layout::Node::with_children(node, vec![content.translate(offset)]).translate(self.offset)
    }

    pub(super) fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn iced_native::Clipboard,
        shell: &mut Shell<'_, Message>,
        index: usize,
        on_event: &dyn Fn(super::Event) -> Message,
    ) -> event::Status {
        let bounds = layout.bounds();
        let content_bounds = layout.children().next().unwrap().bounds();
        let in_bounds =
            bounds.contains(cursor_position) && !content_bounds.contains(cursor_position);

        let state = tree.state.downcast_mut::<State>();

        if let State::Translating { started_at, offset } = state {
            if let iced_native::Event::Mouse(event) = event {
                match event {
                    mouse::Event::CursorMoved { .. } => {
                        *offset = cursor_position - *started_at;
                        return event::Status::Captured;
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        shell.publish((on_event)(Event::NodeMoved {
                            index,
                            offset: self.offset + *offset,
                        }));
                        *state = in_bounds.then_some(State::Hovered).unwrap_or(State::Idle);
                    }
                    _ => {}
                }
            }

            event::Status::Ignored
        } else {
            let status = self.content.as_widget_mut().on_event(
                tree.children.first_mut().unwrap(),
                event.clone(),
                layout.children().next().unwrap(),
                cursor_position,
                renderer,
                clipboard,
                shell,
            );

            if matches!(status, event::Status::Ignored) {
                if let iced_native::Event::Mouse(event) = event {
                    match event {
                        mouse::Event::CursorMoved { .. }
                            if in_bounds && matches!(*state, State::Idle) =>
                        {
                            *state = State::Hovered;
                            return event::Status::Captured;
                        }
                        mouse::Event::CursorMoved { .. }
                            if !in_bounds && matches!(*state, State::Hovered) =>
                        {
                            *state = State::Idle;
                            return event::Status::Captured;
                        }
                        mouse::Event::ButtonPressed(mouse::Button::Left)
                            if matches!(*state, State::Hovered) =>
                        {
                            *state = State::Translating {
                                started_at: cursor_position,
                                offset: Vector::default(),
                            };
                            return event::Status::Captured;
                        }
                        _ => {}
                    }
                }

                event::Status::Ignored
            } else {
                status
            }
        }
    }

    pub(super) fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_native::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let appearance = theme.appearance(self.style);

        let draw = |renderer: &mut Renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_radius: appearance.border_radius,
                    border_width: appearance.border_width,
                    border_color: appearance.border_color,
                },
                appearance
                    .background
                    .unwrap_or_else(|| Color::TRANSPARENT.into()),
            );
            self.content.as_widget().draw(
                tree.children.first().unwrap(),
                renderer,
                theme,
                &renderer::Style {
                    text_color: appearance.text_color.unwrap_or(style.text_color),
                },
                layout.children().next().unwrap(),
                cursor_position,
                viewport,
            )
        };

        if let State::Translating { offset, .. } = state {
            renderer.with_translation(*offset, |renderer| {
                draw(renderer);
            });
        } else {
            draw(renderer);
        }
    }

    pub(super) fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        match state {
            State::Idle => mouse::Interaction::default(),
            State::Hovered => mouse::Interaction::Grab,
            State::Translating { .. } => mouse::Interaction::Grabbing,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub text_color: Option<Color>,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            text_color: None,
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

pub trait StyleSheet {
    type Style: Default + Copy;

    fn appearance(&self, style: Self::Style) -> Appearance;
}
