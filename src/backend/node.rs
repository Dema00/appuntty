#[derive(Debug, PartialEq, Eq)]
pub struct UUID<'p> {
    parent: &'p UUID<'p>,
    id: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeElement<'s>{
    Word(&'s str),
    TempBlob((String, Vec<u32>)),
    TempRef(Vec<u32>),
    Property(NodeProperty)
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeContent<'p> {
    Text(String),
    Reference(UUID<'p>),
    Blob((String, Box<Node<'p>>)),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node<'p> {
    parent: &'p Node<'p>,
    uuid: UUID<'p>,
    cont: Vec<NodeContent<'p>>,
    sons: Vec<Box<Node<'p>>>,
    prop: Vec<NodeProperty>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeProperty {
    Color,
    Rbind(Vec<u32>),
    Blob,
}
