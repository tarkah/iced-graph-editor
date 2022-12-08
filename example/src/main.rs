use iced::widget::{button, column, container, text};
use iced::{executor, theme, Application, Command, Element, Length, Settings, Theme, Vector};

use iced_graph_editor::widget::graph;
use iced_graph_editor::widget::graph::editor;

use self::node::Node;

mod node;

fn main() {
    App::run(Settings {
        antialiasing: true,
        ..Default::default()
    })
    .unwrap()
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Graph(editor::Event),
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
                edges: vec![1],
            },
            Node {
                kind: node::Kind::B,
                offset: Vector::new(150.0, 100.0),
                edges: vec![2, 3],
            },
            Node {
                kind: node::Kind::C,
                offset: Vector::new(350.0, 25.0),
                edges: vec![3],
            },
            Node {
                kind: node::Kind::D,
                offset: Vector::new(500.0, 200.0),
                edges: vec![],
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
                editor::Event::NodeMoved { index, offset } => {
                    self.nodes[index].offset = offset;

                    Command::none()
                }
                editor::Event::Scaled(scaling, translation) => {
                    self.scaling = scaling;
                    self.translation = translation;

                    Command::none()
                }
                editor::Event::Translated(translation) => {
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
                self.nodes.iter_mut().for_each(|node| {
                    node.edges = std::mem::take(&mut node.edges)
                        .into_iter()
                        .filter(|i| *i != index)
                        .map(|i| if i > index { i - 1 } else { i })
                        .collect();
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
            .map(|node| graph::Node::new(node_content(node.kind), node.offset, node.edges.clone()))
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
