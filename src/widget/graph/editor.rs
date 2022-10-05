use iced::{Background, Color, Length, Point, Rectangle, Size, Vector};
use iced_graphics::Renderer;
use iced_native::widget::Tree;
use iced_native::{event, layout, renderer, Element, Layout, Renderer as _, Widget};

use super::{node, Node};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    NodeMoved { index: usize, offset: Vector },
}

pub struct Editor<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet + node::StyleSheet,
{
    nodes: Vec<Node<'a, Message, Renderer>>,
    max_node_size: Size,
    on_event: Box<dyn Fn(Event) -> Message + 'a>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Editor<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet + node::StyleSheet,
{
    pub fn new(
        nodes: Vec<Node<'a, Message, Renderer>>,
        on_event: impl Fn(Event) -> Message + 'a,
    ) -> Self {
        Self {
            nodes,
            max_node_size: Size::new(300.0, 300.0),
            on_event: Box::new(on_event),
            style: Default::default(),
        }
    }

    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Backend, Theme> Widget<Message, Renderer<Backend, Theme>>
    for Editor<'a, Message, Renderer<Backend, Theme>>
where
    Backend: iced_graphics::Backend,
    Theme: StyleSheet + node::StyleSheet,
{
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
    ) -> iced::event::Status {
        self.nodes
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .enumerate()
            .map(|(index, ((node, state), layout))| {
                node.on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                    index,
                    &self.on_event,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
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
                        cursor_position,
                        viewport,
                    )
                });

            {
                use iced::widget::canvas::{Frame, Path, Stroke};

                self.nodes
                    .iter()
                    .enumerate()
                    .for_each(|(from_index, from)| {
                        if let Some(to_index) = from.edge {
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

                                let mut frame = Frame::new(viewport.size());

                                let start = Point {
                                    x: from_bounds.x + from_bounds.width,
                                    y: from_bounds.center_y(),
                                };
                                let end = Point {
                                    x: to_bounds.x,
                                    y: to_bounds.center_y(),
                                };

                                let path = Path::line(start, end);

                                frame.stroke(&path, Stroke::default());

                                let primitive = frame.into_geometry().into_primitive();
                                renderer.draw_primitive(primitive);
                            }
                        }
                    });
            }
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
        self.nodes
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((node, state), layout)| {
                node.mouse_interaction(state, layout, cursor_position, viewport, renderer)
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
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
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
