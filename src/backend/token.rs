#[derive(Clone, PartialEq)]
pub enum Token {
    Anchor,
    Text(String),
    AnchorID(i32),
    Blob(String)
}