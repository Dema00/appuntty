use core::fmt;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::backend::parser::{RefSetterClosure, VecID};

#[derive(Debug, Clone)]
pub struct UUID {
    pub parent: Weak<RefCell<UUID>>,
    pub id: RefCell<usize>,
}

impl PartialEq for UUID {
    fn eq(&self, other: &Self) -> bool {
        self.parent.upgrade() == other.parent.upgrade() && self.id == other.id
    }
}

impl Eq for UUID {}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_vec_id().iter().fold(String::new(), |string, id| {
                if string.is_empty() {
                    format!("{}", id)
                } else {
                    format!("{}.{}", string, id)
                }
            })
        )
    }
}

impl UUID {
    pub fn new(parent: Option<SRef<UUID>>, id: usize) -> SRef<UUID> {
        Rc::new(RefCell::new(UUID {
            parent: parent.map_or(Weak::new(), |parent| Rc::downgrade(&Rc::clone(&parent))),
            id: RefCell::new(id),
        }))
    }

    fn update_id(&self, new_id: usize) {
        *self.id.borrow_mut() = new_id;
    }

    pub fn to_vec_id(&self) -> Vec<usize> {
        let mut p = self.parent.upgrade();
        let mut id = vec![self.id.clone().into_inner()];

        while let Some(pin) = p {
            id.push(pin.borrow().id.clone().into_inner());
            p = pin.borrow().parent.upgrade();
        }

        id.reverse();
        id
    }
}

//type aliases for legibility

pub type SRef<T> = Rc<RefCell<T>>; //Reference counted refcell
pub type WSRef<T> = Weak<RefCell<T>>; //Weakly reference counted refcell
pub type HRef<T> = Rc<Box<RefCell<T>>>; //Reference counted heap refcell
pub type WHRef<T> = Weak<Box<RefCell<T>>>; //Weakly reference counted heap refcell

//Da rimuovere, aggiungere Futures per la gestione degli UUID linkati
#[derive(Debug, PartialEq, Eq)]
pub enum NodeElement<'s> {
    Word(&'s str),
    TempBlob((String, Vec<usize>)),
    TempRef(Vec<usize>),
    Property(NodeProperty),
}

#[derive(Debug, Clone)]
pub enum NodeContent {
    Text(String),
    Reference(WSRef<UUID>),
    Blob((String, WSRef<UUID>)),
}

#[derive(Debug)]
pub struct Node {
    pub parent: WHRef<Node>,
    pub uuid: SRef<UUID>,
    pub cont: RefCell<Vec<NodeContent>>,
    pub sons: RefCell<Vec<HRef<Node>>>,
    pub prop: RefCell<Vec<NodeProperty>>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        fn print_uuid(uuid: &WSRef<UUID>) -> String {
            match uuid.upgrade() {
                Some(uuid) => format!("{}",uuid.borrow()),
                None => format!("No Reference)"),
            }
        }

        write!(f, "{}", "- ")?;

        write!(f, "({}) ", self.uuid.borrow())?;

        for content in self.cont.borrow().iter() {
            match content {
                NodeContent::Text(text) => write!(f, "{} ", text),
                NodeContent::Blob((text, uuid)) => write!(f, "{{{}}}({}) ", text, print_uuid(uuid)),
                NodeContent::Reference(uuid) => write!(f, "#({}) ", print_uuid(uuid)),
            }?
        }

        for property in self.prop.borrow().iter() {
            match property {
                NodeProperty::Blob => write!(f, "<blob>"),
                NodeProperty::Color => write!(f, "<colored>"),
                NodeProperty::Rbind(_) => write!(f, "<rbind>"),
            }?
        }

        write!(f, "\n")?;

        for son in self.sons.borrow().iter() {
            write!(
                f,
                "{}{}",
                (0..self.get_depth()).map(|_| " ").collect::<String>(),
                son.borrow()
            )?;
        }
        Ok(())
    }
}

impl Node {
    pub fn new(parent: Option<HRef<Node>>) -> HRef<Self> {
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

    pub fn replace_content(&self, id: usize, new_content: NodeContent) {
        self.cont.borrow_mut()[id] = new_content;
    }

    pub fn check_and_rectify_wanted_status(
        &self,
        wanted_uuids: &mut HashMap<(VecID), RefCell<Vec<RefSetterClosure>>>,
    ) {
        match wanted_uuids.get(&self.uuid.borrow().to_vec_id()) {
            Some(vec) => {vec.borrow().iter().for_each(|closure| closure(Rc::clone(&self.uuid)));},
            None => (),
        }
    }

    pub fn get_cont_len(&self) -> usize {
        self.cont.borrow().len()
    }

    pub fn append_contents(&self, mut contents: Vec<NodeContent>) {
        self.cont.borrow_mut().append(&mut contents);
    }

    pub fn push_content(&self, content: NodeContent) {
        self.cont.borrow_mut().push(content);
    }

    pub fn push_property(&self, property: NodeProperty) {
        self.prop.borrow_mut().push(property);
    }

    pub fn push_child(&self, child: HRef<Node>) {
        self.sons.borrow_mut().push(child)
    }

    pub fn insert_child(&self, idx: usize, child: HRef<Node>) {
        self.sons.borrow_mut().insert(idx, child);
        self.update_child_ids()
    }

    pub fn get_depth(&self) -> usize {
        let mut p = self.parent.upgrade();
        let mut depth = 0;

        while p.is_some() {
            p = p.unwrap().borrow().parent.upgrade();
            depth += 1;
        }
        depth + 1
    }

    pub fn update_child_ids(&self) {
        self.sons
            .borrow_mut()
            .iter_mut()
            .fold(0, |count: usize, a| {
                a.borrow().uuid.borrow().update_id(count);
                count + 1
            });
    }

    pub fn go_up(&self, mut amount: usize) -> Option<HRef<Node>> {
        let mut p = self.parent.upgrade();

        while let Some(pin) = p {
            if amount == 0 {
                return Some(pin);
            }
            amount -= 1;
            p = pin.borrow().parent.upgrade()
        }

        None
    }

    pub fn go_down(&self, addr: Vec<usize>) -> Option<HRef<Node>> {
        if addr.is_empty() {
            return None;
        }
        if addr.len() == 1 {
            self.sons
                .borrow()
                .get(addr[0])
                .map_or(None, |node| Some(Rc::clone(node)))
        } else {
            self.sons
                .borrow()
                .get(addr[0])
                .map_or(None, |node| node.borrow().go_down(addr[1..].to_vec()))
        }
    }

    pub fn search_by_uuid(&self, uuid: SRef<UUID>) -> Option<HRef<Node>> {
        let v_id_other = uuid.borrow().to_vec_id();
        self.search_by_vec_id(&v_id_other)
    }

    pub fn search_by_vec_id(&self, addr: &Vec<usize>) -> Option<HRef<Node>> {
        let v_id_self = self.uuid.borrow().to_vec_id();

        let shared_addr_len: usize =
            v_id_self
                .iter()
                .zip(addr.clone())
                .fold(0, |c, (&s, o)| if s == o { c + 1 } else { c });

        let distance_from_shared_addr = (shared_addr_len).abs_diff(v_id_self.len());

        let starting_node = self.go_up(distance_from_shared_addr)?;

        let return_node = starting_node
            .borrow()
            .go_down(addr[shared_addr_len..].to_vec());

        return_node
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeProperty {
    Color,
    Rbind(Vec<usize>),
    Blob,
}
