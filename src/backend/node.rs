pub struct UUID<'p> {
    parent: &'p UUID<'p>,
    id: i32,
}

pub enum NodeContent<'p> {
    Text(String),
    Reference(UUID<'p>),
    Blob((String, Box<Node<'p>>)),
}

pub struct Node<'p> {
    parent: &'p Node<'p>,
    uuid: UUID<'p>,
    cont: Vec<NodeContent<'p>>,
    sons: Vec<Box<Node<'p>>>,
    prop: Vec<NodeProperty>,
}

pub enum NodeProperty {
    Color,
    Rbind,
    Blob,
}
