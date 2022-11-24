use iced::{Background, Color, Length, Point, Rectangle, Size, Vector};
use iced_graphics::{Renderer, Transformation};
use iced_native::widget::{tree, Tree};
use iced_native::{event, layout, mouse, renderer, Element, Layout, Renderer as _, Widget};

use super::{node, Node};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    NodeMoved { index: usize, offset: Vector },
    Scaled(f32, Vector),
    Translated(Vector),
}

#[derive(Debug, Clone, Copy, Default)]
enum Interaction {
    #[default]
    Idle,
    Translating {
        started_at: Point,
        offset: Vector,
    },
}

impl Interaction {
    fn offset(&self) -> Vector {
        match self {
            Interaction::Idle => Vector::default(),
            Interaction::Translating { offset, .. } => *offset,
        }
    }
}

pub struct Editor<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet + node::StyleSheet,
{
    nodes: Vec<Node<'a, Message, Renderer>>,
    scaling: f32,
    translation: Vector,
    max_node_size: Size,
    on_event: Box<dyn Fn(Event) -> Message + 'a>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Editor<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet + node::StyleSheet,
{
    const MIN_SCALING: f32 = 0.1;
    const MAX_SCALING: f32 = 5.0;

    pub fn new(
        nodes: Vec<Node<'a, Message, Renderer>>,
        on_event: impl Fn(Event) -> Message + 'a,
    ) -> Self {
        Self {
            nodes,
            scaling: 1.0,
            translation: Vector::new(0.0, 0.0),
            max_node_size: Size::new(300.0, 300.0),
            on_event: Box::new(on_event),
            style: Default::default(),
        }
    }

    pub fn style(self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        Self {
            style: style.into(),
            ..self
        }
    }

    pub fn scaling(self, scaling: f32) -> Self {
        Self { scaling, ..self }
    }

    pub fn translation(self, translation: Vector) -> Self {
        Self {
            translation,
            ..self
        }
    }

    fn transformation(&self) -> glam::Mat4 {
        (Transformation::identity()
            * Transformation::scale(self.scaling, self.scaling)
            * Transformation::translate(self.translation.x, self.translation.y))
        .into()
    }

    fn transform_cursor(&self, cursor_position: Point) -> Point {
        let Point { x, y } = cursor_position;

        let glam::Vec3 { x, y, .. } = self
            .transformation()
            .inverse()
            .transform_point3(glam::Vec3::new(x, y, 1.0));

        Point::new(x, y)
    }
}

impl<'a, Message, Backend, Theme> Widget<Message, Renderer<Backend, Theme>>
    for Editor<'a, Message, Renderer<Backend, Theme>>
where
    Backend: iced_graphics::Backend,
    Theme: StyleSheet + node::StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Interaction>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Interaction::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.nodes
            .iter()
            .map(|node| Tree {
                tag: node.tag(),
                state: node.state(),
                children: node.children(),
            })
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children_custom(
            &self.nodes,
            |state, node| node.diff(state),
            |node| Tree {
                tag: node.tag(),
                state: node.state(),
                children: node.children(),
            },
        )
    }

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(&self, renderer: &Renderer<Backend, Theme>, limits: &layout::Limits) -> layout::Node {
        layout::Node::with_children(
            limits.fill(),
            self.nodes
                .iter()
                .map(|node| {
                    node.layout(
                        renderer,
                        &layout::Limits::new(Size::ZERO, self.max_node_size),
                    )
                })
                .collect(),
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer<Backend, Theme>,
        clipboard: &mut dyn iced_native::Clipboard,
        shell: &mut iced_native::Shell<'_, Message>,
    ) -> event::Status {
        let interaction = tree.state.downcast_mut::<Interaction>();

        let bounds = layout.bounds();
        let contains_cursor = bounds.contains(cursor_position);

        let transformed_cursor = self.transform_cursor(cursor_position);

        let status = self
            .nodes
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .enumerate()
            .map(|(index, ((node, state), layout))| {
                node.on_event(
                    state,
                    event.clone(),
                    layout,
                    transformed_cursor,
                    renderer,
                    clipboard,
                    shell,
                    index,
                    &self.on_event,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge);

        if matches!(status, event::Status::Ignored) && contains_cursor {
            match event {
                event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    *interaction = Interaction::Translating {
                        started_at: cursor_position,
                        offset: Vector::default(),
                    };
                    return event::Status::Captured;
                }
                event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if let Interaction::Translating { offset, .. } = interaction {
                        shell.publish((self.on_event)(Event::Translated(
                            self.translation + *offset,
                        )));

                        *interaction = Interaction::Idle;
                        return event::Status::Captured;
                    }
                }
                event::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    if let Interaction::Translating { started_at, offset } = interaction {
                        *offset = (position - *started_at) * (1.0 / self.scaling);
                        return event::Status::Captured;
                    }
                }
                event::Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        if y < 0.0 && self.scaling > Self::MIN_SCALING
                            || y > 0.0 && self.scaling < Self::MAX_SCALING
                        {
                            let old_scaling = self.scaling;

                            let scaling = (self.scaling * (1.0 + y / 15.0))
                                .max(Self::MIN_SCALING)
                                .min(Self::MAX_SCALING);
                            let factor = scaling - old_scaling;

                            let translation = self.translation
                                - Vector::new(
                                    cursor_position.x * factor / (old_scaling * old_scaling),
                                    cursor_position.y * factor / (old_scaling * old_scaling),
                                );

                            shell.publish((self.on_event)(Event::Scaled(scaling, translation)));

                            return event::Status::Captured;
                        }
                    }
                },
                _ => {}
            }

            event::Status::Ignored
        } else {
            status
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer<Backend, Theme>,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let interaction = tree.state.downcast_ref::<Interaction>();

        let transformed_cursor = self.transform_cursor(cursor_position);

        let appearance = <Theme as StyleSheet>::appearance(theme, self.style);

        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border_width: appearance.border_width,
                border_color: appearance.border_color,
                border_radius: appearance.border_radius,
            },
            appearance
                .background
                .unwrap_or_else(|| Color::TRANSPARENT.into()),
        );

        let pad = |rect: Rectangle, padding: f32| Rectangle {
            x: rect.x + padding,
            y: rect.y + padding,
            width: rect.width - padding * 2.0,
            height: rect.height - padding * 2.0,
        };

        let padded_bounds = pad(layout.bounds(), 1.0);

        renderer.with_layer(padded_bounds, |renderer| {
            renderer.with_translation(self.translation + interaction.offset(), |renderer| {
                renderer.with_scale(self.scaling, |renderer| {
                    self.nodes
                        .iter()
                        .zip(&tree.children)
                        .zip(layout.children())
                        .for_each(|((node, state), layout)| {
                            node.draw(
                                state,
                                renderer,
                                theme,
                                style,
                                layout,
                                transformed_cursor,
                                viewport,
                            )
                        });
                });
            });

            let frame_offset = Vector::new(padded_bounds.x, padded_bounds.y);
            renderer.with_translation(frame_offset, |renderer| {
                use iced::widget::canvas::{Frame, Path, Stroke};

                self.nodes
                    .iter()
                    .enumerate()
                    .for_each(|(from_index, from)| {
                        for to_index in from.edges.iter().copied() {
                            if self.nodes.get(to_index).is_some() {
                                let from_state = tree
                                    .children
                                    .get(from_index)
                                    .unwrap()
                                    .state
                                    .downcast_ref::<node::State>();
                                let to_state = tree
                                    .children
                                    .get(to_index)
                                    .unwrap()
                                    .state
                                    .downcast_ref::<node::State>();

                                let from_bounds = from_state.adjusted_bounds(
                                    layout.children().nth(from_index).unwrap().bounds(),
                                );
                                let to_bounds = to_state.adjusted_bounds(
                                    layout.children().nth(to_index).unwrap().bounds(),
                                );

                                let mut frame = Frame::new(padded_bounds.size());

                                let transform_point = |point: Point| {
                                    let translated =
                                        point + self.translation + interaction.offset();

                                    Point {
                                        x: translated.x * self.scaling,
                                        y: translated.y * self.scaling,
                                    } - frame_offset
                                };

                                let start_untransformed = Point {
                                    x: (from_bounds.x + from_bounds.width),
                                    y: from_bounds.center_y(),
                                };
                                let start = transform_point(start_untransformed);
                                let end_untransformed = Point {
                                    x: to_bounds.x,
                                    y: to_bounds.center_y(),
                                };
                                let end = transform_point(end_untransformed);

                                let path = Path::new(|p| {
                                    let control_scale =
                                        ((end_untransformed.x - start_untransformed.x) / 2.0)
                                            .max(30.0)
                                            * self.scaling;
                                    let control_a = Point {
                                        x: start.x + control_scale,
                                        y: start.y,
                                    };
                                    let control_b = Point {
                                        x: end.x - control_scale,
                                        y: end.y,
                                    };

                                    p.move_to(start);
                                    p.bezier_curve_to(control_a, control_b, end);
                                });

                                frame.stroke(
                                    &path,
                                    Stroke::default()
                                        .with_width(appearance.connector_width * self.scaling)
                                        .with_color(appearance.connector_color),
                                );

                                let primitive = frame.into_geometry().into_primitive();
                                renderer.draw_primitive(primitive);
                            }
                        }
                    });
            });
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer<Backend, Theme>,
    ) -> iced_native::mouse::Interaction {
        let transformed_cursor = self.transform_cursor(cursor_position);

        self.nodes
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((node, state), layout)| {
                node.mouse_interaction(state, layout, transformed_cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }
}

impl<'a, Message, Backend, Theme> From<Editor<'a, Message, Renderer<Backend, Theme>>>
    for Element<'a, Message, Renderer<Backend, Theme>>
where
    Backend: iced_graphics::Backend + 'a,
    Theme: StyleSheet + node::StyleSheet + 'a,
    Message: 'a,
{
    fn from(editor: Editor<'a, Message, Renderer<Backend, Theme>>) -> Self {
        Element::new(editor)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub connector_width: f32,
    pub connector_color: Color,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            connector_width: 1.0,
            connector_color: Color::BLACK,
        }
    }
}

pub trait StyleSheet {
    type Style: Default + Copy;

    fn appearance(&self, style: Self::Style) -> Appearance;
}
