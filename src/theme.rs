use iced_style::Theme;

use crate::widget::graph::{editor, node};

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

#[derive(Debug, Clone, Copy, Default)]
pub enum Editor {
    #[default]
    Default,
}

impl editor::StyleSheet for Theme {
    type Style = Node;

    fn appearance(&self, style: Self::Style) -> editor::Appearance {
        match style {
            Node::Default => editor::Appearance {
                background: Some(self.extended_palette().background.weak.color.into()),
                border_radius: 0.0,
                border_width: 1.0,
                border_color: self.extended_palette().background.strong.color,
                connector_width: 2.0,
                connector_color: self.palette().text,
            },
        }
    }
}
