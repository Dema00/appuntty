use super::{WSRef, UUID};

#[derive(Debug, Clone)]
pub enum NodeContent {
    Text(String),
    Reference(Reference),
    Blob(Blob),
}

#[derive(Debug, Clone)]
pub struct Blob {
    pub text: String,
    pub link: WSRef<UUID>,
}

impl Blob {
    pub fn new(text: String, link: WSRef<UUID>) -> Self {
        Blob { text, link }
    }
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub link: WSRef<UUID>,
}

impl Reference {
    pub fn new(link: WSRef<UUID>) -> Self {
        Reference { link }
    }
}