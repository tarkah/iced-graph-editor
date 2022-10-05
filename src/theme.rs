pub use iced::theme::Container;
pub use iced::Theme;

use crate::widget::graph::node;

#[derive(Debug, Clone, Copy, Default)]
pub enum Node {
    #[default]
    Default,
}

impl node::StyleSheet for Theme {
    type Style = Node;

    fn appearance(&self, style: Self::Style) -> node::Appearance {
        match style {
            Node::Default => node::Appearance {
                text_color: Some(self.palette().text),
                background: Some(self.palette().background.into()),
                border_radius: 3.0,
                border_width: 1.0,
                border_color: self.extended_palette().background.strong.color,
            },
        }
    }
}
