use std::{
    borrow::BorrowMut,
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Debug, Clone)]
pub struct UUID {
    pub parent: Weak<RefCell<UUID>>,
    pub id: usize,
}

impl UUID {
    fn new(parent: Option<Rc<RefCell<UUID>>>, id: usize) -> Rc<RefCell<UUID>> {
        Rc::new(RefCell::new(UUID {
            parent: parent.map_or(Weak::new(), |parent| Rc::downgrade(&Rc::clone(&parent))),
            id,
        }))
    }
}
//Da rimuovere, aggiungere Futures per la gestione degli UUID linkati
#[derive(Debug, PartialEq, Eq)]
pub enum NodeElement<'s> {
    Word(&'s str),
    TempBlob((String, Vec<usize>)),
    TempRef(Vec<usize>),
    Property(NodeProperty),
}

#[derive(Debug)]
pub enum NodeContent {
    Text(String),
    Reference(Weak<UUID>),
    Blob((String, Weak<UUID>)),
}

#[derive(Debug)]
pub struct Node {
    pub parent: Weak<Box<RefCell<Node>>>,
    pub uuid: Rc<RefCell<UUID>>,
    pub cont: RefCell<Vec<NodeContent>>,
    pub sons: RefCell<Vec<Rc<Box<RefCell<Node>>>>>,
    pub prop: RefCell<Vec<NodeProperty>>,
}

impl Node {
    fn new(parent: Option<Rc<Box<RefCell<Node>>>>) -> Rc<Box<RefCell<Self>>> {
        Rc::new(Box::new(RefCell::new(Node {
            parent: parent
                .clone()
                .map_or(Weak::new(), |parent| Rc::downgrade(&Rc::clone(&parent))),
            uuid: parent.map_or(UUID::new(None, 0), |parent| {
                UUID::new(Some(Rc::clone(&parent.borrow().uuid)),
                parent.borrow().sons.borrow().len()
            )
            }),
            cont: RefCell::new(Vec::new()),
            sons: RefCell::new(Vec::new()),
            prop: RefCell::new(Vec::new()),
        })))
    }

    fn add_child(&self, child: Rc<Box<RefCell<Node>>>) {
        self.sons.borrow_mut().push(child)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeProperty {
    Color,
    Rbind(Vec<usize>),
    Blob,
}
