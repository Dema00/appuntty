pub mod backend;
pub mod frontend;
use backend::{
    node::{SRef, UUID},
    parser::{node, VecID},
};
use std::{cell::RefCell, collections::HashMap, fs};

fn main() {
    let file =
        fs::read_to_string("text_example.md").expect("Should have been able to read the file");

    let mut wanted_uuids = HashMap::<VecID, RefCell<Vec<Box<dyn Fn(SRef<UUID>) -> ()>>>>::new();

    let file = node(file.as_str(), None, &mut wanted_uuids)
        .expect("Error during parsing")
        .1;

    println!("{}", file.borrow());
}
