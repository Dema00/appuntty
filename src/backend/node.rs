pub struct UUID<'s> { 
    parent: &'s UUID<'s>, 
    id: i32,
}

enum NodeContents<'s> {
    Text(String),
    Reference(UUID<'s>),
    Blob((String,Box<Node<'s>>)),
}

pub struct Node<'s> {
    parent: &'s Node<'s>,
    uuid: UUID<'s>,
    cont: Vec<NodeContents<'s>>,
    sons: Vec<Box<Node<'s>>>,
}