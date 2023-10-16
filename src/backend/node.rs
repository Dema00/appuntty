#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UUID<'p> {
    pub parent: Option<&'p UUID<'p>>,
    pub id: u32,
}


//Da rimuovere, aggiungere Futures per la gestione degli UUID linkati
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
    Blob((String, UUID<'p>)),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node<'p> {
    pub parent: Option<&'p Node<'p>>,
    pub uuid: UUID<'p>,
    pub cont: Vec<NodeContent<'p>>,
    pub sons: Vec<Box<Node<'p>>>,
    pub prop: Vec<NodeProperty>,
}

impl<'p> Node<'p> {
    pub fn new(parent: &'p Node) -> Self {
        Node { 
            parent: Some(parent),
            uuid: UUID {
                parent: Some(&parent.uuid),
                id: parent.sons.len() as u32
            },
            cont: Vec::new(),
            sons: Vec::new(),
            prop: Vec::new() 
        }
    }

    pub fn root(&self) -> &Self {
        self.parent.map_or(&self, |s| s.root())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeProperty {
    Color,
    Rbind(Vec<u32>),
    Blob,
}
