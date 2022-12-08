use iced::Vector;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: Kind,
    pub offset: Vector,
    pub edges: Vec<usize>,
}
