use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Debug, Clone)]
pub struct UUID {
    pub parent: Weak<RefCell<UUID>>,
    pub id: RefCell<usize>,
}
//type aliases for legibility

type Ref<T> = Rc<RefCell<T>>; //Reference counted refcell
type HRef<T> = Rc<Box<RefCell<T>>>; //Reference counted heap refcell
type WHRef<T> = Weak<Box<RefCell<T>>>; //Weakly reference counted heap refcell

impl UUID {
    fn new(parent: Option<Ref<UUID>>, id: usize) -> Ref<UUID> {
        Rc::new(RefCell::new(UUID {
            parent: parent.map_or(Weak::new(), |parent| Rc::downgrade(&Rc::clone(&parent))),
            id: RefCell::new(id),
        }))
    }

    fn update_id(&self, new_id: usize) {
        *self.id.borrow_mut() = new_id;
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
    pub parent: WHRef<Node>,
    pub uuid: Ref<UUID>,
    pub cont: RefCell<Vec<NodeContent>>,
    pub sons: RefCell<Vec<HRef<Node>>>,
    pub prop: RefCell<Vec<NodeProperty>>,
}

impl Node {
    fn new(parent: Option<HRef<Node>>) -> HRef<Self> {
        Rc::new(Box::new(RefCell::new(Node {
            parent: parent
                .clone()
                .map_or(Weak::new(), |parent| Rc::downgrade(&Rc::clone(&parent))),
            uuid: parent.map_or(UUID::new(None, 0), |parent| {
                UUID::new(
                    Some(Rc::clone(&parent.borrow().uuid)),
                    parent.borrow().sons.borrow().len(),
                )
            }),
            cont: RefCell::new(Vec::new()),
            sons: RefCell::new(Vec::new()),
            prop: RefCell::new(Vec::new()),
        })))
    }

    fn push_child(&self, child: HRef<Node>) {
        self.sons.borrow_mut().push(child)
    }

    fn insert_child(&self, idx: usize, child: HRef<Node>) {
        self.sons.borrow_mut().insert(idx, child);
        self.update_child_ids()
    }

    fn update_child_ids(&self) {
        self.sons
            .borrow_mut()
            .iter_mut()
            .fold(0, |count: usize, a| {
                a.borrow().uuid.borrow().update_id(count);
                count + 1
            });
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeProperty {
    Color,
    Rbind(Vec<usize>),
    Blob,
}
