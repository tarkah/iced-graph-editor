use iced::widget::{button, column, container, text};
use iced::{executor, Application, Command, Element, Length, Settings, Theme, Vector};

use self::node::Node;
use self::widget::graph;

mod node;
mod theme;
mod widget;

fn main() {
    App::run(Settings {
        antialiasing: true,
        ..Default::default()
    })
    .unwrap()
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Graph(graph::Event),
    ToggleTheme,
    DeleteNode(usize),
}

struct App {
    nodes: Vec<Node>,
    scaling: f32,
    translation: Vector,
    theme: Theme,
}

impl Application for App {
    type Executor = executor::Default;
    type Theme = Theme;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let nodes = vec![
            Node {
                kind: node::Kind::A,
                offset: Vector::new(50.0, 50.0),
                edge: Some(1),
            },
            Node {
                kind: node::Kind::B,
                offset: Vector::new(150.0, 100.0),
                edge: Some(3),
            },
            Node {
                kind: node::Kind::C,
                offset: Vector::new(150.0, 300.0),
                edge: Some(3),
            },
            Node {
                kind: node::Kind::D,
                offset: Vector::new(350.0, 200.0),
                edge: None,
            },
        ];

        (
            App {
                nodes,
                scaling: 1.0,
                translation: Vector::new(0.0, 0.0),
                theme: Theme::Light,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Iced Graph Editor".into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Graph(event) => match event {
                graph::Event::NodeMoved { index, offset } => {
                    self.nodes[index].offset = offset;

                    Command::none()
                }
                graph::Event::Scaled(scaling, translation) => {
                    self.scaling = scaling;
                    self.translation = translation;

                    Command::none()
                }
                graph::Event::Translated(translation) => {
                    self.translation = translation;

                    Command::none()
                }
            },
            Message::ToggleTheme => {
                match &self.theme {
                    Theme::Light => self.theme = Theme::Dark,
                    Theme::Dark => self.theme = Theme::Light,
                    Theme::Custom(_) => {}
                }

                Command::none()
            }
            Message::DeleteNode(index) => {
                self.nodes.remove(index);
                self.nodes.iter_mut().for_each(|node| match &mut node.edge {
                    edge if *edge == Some(index) => {
                        edge.take();
                    }
                    edge if *edge > Some(index) => {
                        *edge = edge.map(|index| index - 1);
                    }
                    _ => {}
                });

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let node_content = |kind: node::Kind| -> Element<_> {
            match kind {
                node::Kind::A => text("Node A").into(),
                node::Kind::B => column![text("Node B"), text("Some description...")]
                    .spacing(5)
                    .into(),
                node::Kind::C => column![
                    text("Node C"),
                    button(text("Delete")).on_press(Message::DeleteNode(2))
                ]
                .spacing(5)
                .into(),
                node::Kind::D => column![
                    text("Node D"),
                    button(text("Toggle Theme")).on_press(Message::ToggleTheme)
                ]
                .spacing(5)
                .into(),
            }
        };

        let nodes = self
            .nodes
            .iter()
            .map(|node| graph::Node::new(node_content(node.kind), node.offset, node.edge))
            .collect();

        container(
            container(
                graph::Editor::new(nodes, Message::Graph)
                    .scaling(self.scaling)
                    .translation(self.translation),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::Container::Box),
        )
        .padding(50)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
