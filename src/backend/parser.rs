extern crate nom;

use crate::backend::node::node_content::NodeContent;

use super::node::{HRef, Node, NodeElement, NodeProperty, SRef, WSRef, UUID, node_content::{Blob, Reference}};

use core::panic;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
    str::FromStr,
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::{
        complete::char,
        complete::{digit1, multispace0, multispace1, newline},
    },
    combinator::{map, map_res, peek},
    //error::{ParseError},
    multi::{many0, many0_count, separated_list1},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
    Parser,
};

//Data Structures

pub type VecID = Vec<usize>;

//Text Parser Elements

fn parse_numbers(input: &str) -> IResult<&str, usize> {
    map_res(digit1, usize::from_str).parse(input)
}

fn parse_id(input: &str) -> IResult<&str, VecID> {
    delimited(tag("("), separated_list1(tag("."), parse_numbers), tag(")")).parse(input)
}

fn uuid(input: &str) -> IResult<&str, VecID> {
    preceded(tag("#"), parse_id).parse(input)
}

fn blob(input: &str) -> IResult<&str, (String, Vec<usize>)> {
    pair(
        delimited(
            tag("{"),
            map(take_until("}"), |str| String::from(str)),
            tag("}"),
        ),
        parse_id,
    )
    .parse(input)
}

fn prop_blob(input: &str) -> IResult<&str, NodeProperty> {
    map(tag("blob"), |_| NodeProperty::Blob).parse(input)
}

fn prop_rbind(input: &str) -> IResult<&str, NodeProperty> {
    preceded(
        tag("rbind"),
        delimited(
            tag("["),
            map(separated_list1(tag(","), parse_numbers), |vec| {
                NodeProperty::Rbind(vec)
            }),
            tag("]"),
        ),
    )
    .parse(input)
}

fn property(input: &str) -> IResult<&str, NodeProperty> {
    delimited(tag("<"), alt((prop_blob, prop_rbind)), tag(">")).parse(input)
}

#[rustfmt::skip]
fn word(input: &str) -> IResult<&str, &str> {
    take_until(" ").parse(input)
}

#[rustfmt::skip]
fn node_element(input: &str) -> IResult<&str, NodeElement> {
        alt(
            (
                map(uuid, |ref_id| NodeElement::TempRef(ref_id)),
                map(property, |property| NodeElement::Property(property)),
                map(blob, |blob| NodeElement::TempBlob(blob)),
                map(word, |text| NodeElement::Word(text))
            )
        )
    .parse(input)
}

fn node_content(input: &str) -> IResult<&str, Vec<NodeElement>> {
    preceded(
        char('-'),
        terminated(
            map(isolate_contents, |elements| {
                elements
                    .into_iter()
                    .filter(|el| {
                        if let NodeElement::Word(iel) = el {
                            !iel.is_empty()
                        } else {
                            true
                        }
                    })
                    .collect()
            }),
            newline,
        ),
    )
    .parse(input)
}

fn isolate_contents(input: &str) -> IResult<&str, Vec<NodeElement>> {
    let (input, isolated) = take_until("\n").parse(input)?;
    let (_, elements) = separated_list1(multispace1, node_element).parse(isolated)?;
    Ok((input, elements))
}

fn get_depth<'i>(input: &'i str) -> IResult<&'i str, usize> {
    peek(delimited(many0(newline), many0_count(tag(" ")), tag("-"))).parse(input)
}

pub type RefSetterClosure = Box<dyn Fn(SRef<UUID>) -> ()>;

pub fn node<'i>(
    input: &'i str,
    parent: Option<HRef<Node>>,
    mut wanted_uuids: &mut HashMap<VecID, RefCell<Vec<RefSetterClosure>>>,
) -> IResult<&'i str, HRef<Node>> {
    let (_, depth) = get_depth.parse(input)?;

    let (mut input, contents) = preceded(multispace0, node_content).parse(input)?;

    let new_node = Node::new(parent.map_or(None, |parent_inner| Some(Rc::clone(&parent_inner))));
    populate_node(Rc::clone(&new_node), contents, &mut wanted_uuids);
    new_node
        .borrow()
        .check_and_rectify_wanted_status(wanted_uuids);

    if !input.is_empty() {
        let (_, mut next_depth) = get_depth.parse(input)?;

        while depth < next_depth && !input.is_empty() {
            let child_node;
            (input, child_node) = node(input, Some(Rc::clone(&new_node)), &mut wanted_uuids)?;
            new_node.borrow_mut().push_child(child_node);

            if !input.is_empty() {
                (_, next_depth) = get_depth.parse(input)?;
            }
        }
    }

    IResult::Ok((input, new_node))
}

struct RefOfReference {
    node: HRef<Node>,
    id: usize,
}

impl RefOfReference {
    pub fn new(node: &HRef<Node>) -> Self {
        RefOfReference {
            node: Rc::clone(node),
            id: Rc::clone(node).borrow().get_cont_len(),
        }
    }
}

fn populate_node(
    node: HRef<Node>,
    contents: Vec<NodeElement>,
    wanted_uuids: &mut HashMap<VecID, RefCell<Vec<RefSetterClosure>>>,
) {
    for element in contents {
        match element {
            NodeElement::Word(word) => node
                .borrow_mut()
                .push_content(NodeContent::Text(String::from(word))),
            NodeElement::Property(property) => node.borrow_mut().push_property(property),
            NodeElement::TempBlob((word, addr)) => {
                let new_content =
                    //NodeContent::Blob((word, get_uuid(addr, Rc::clone(&node), wanted_uuids)));
                    NodeContent::Blob(
                        Blob::new(
                            word,
                            get_uuid(addr, Rc::clone(&node), wanted_uuids)
                        )
                    );
                node.borrow_mut().push_content(new_content);
            }
            NodeElement::TempRef(addr) => {
                let new_contet =
                    NodeContent::Reference(
                        Reference::new(get_uuid(addr, Rc::clone(&node), wanted_uuids))
                    );
                node.borrow().push_content(new_contet)
            }
        }
    }
}

fn get_uuid(
    addr_vec: VecID,
    tree: HRef<Node>,
    wanted_uuids: &mut HashMap<VecID, RefCell<Vec<RefSetterClosure>>>,
) -> WSRef<UUID> {
    match tree.borrow().search_by_vec_id(&addr_vec) {
        Some(node) => Rc::downgrade(&node.borrow().uuid),
        None => {
            let ref_ref = RefOfReference::new(&tree);

            let lazy_closure = move |uuid| {
                let cont = ref_ref.node.borrow().cont.borrow()[ref_ref.id].clone();

                match cont {
                    NodeContent::Reference(_) => ref_ref
                        .node
                        .borrow()
                        .replace_content(ref_ref.id, NodeContent::Reference(Reference::new(Rc::downgrade(&uuid)))),

                    NodeContent::Blob(blob) => ref_ref.node.borrow().replace_content(
                        ref_ref.id,
                        NodeContent::Blob(Blob::new(blob.text.clone(), Rc::downgrade(&uuid)))
                    ),

                    _ => panic!("Error while lazily linking UUID: wrong content id"),
                }
            };

            match wanted_uuids.get(&addr_vec) {
                Some(vec) => {
                    vec.borrow_mut().push(Box::new(lazy_closure));
                }
                None => {
                    wanted_uuids.insert(addr_vec, RefCell::new(vec![Box::new(lazy_closure)]));
                }
            };
            Weak::new()
        }
    }
}

/*fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(f: F) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}*/

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use crate::backend::{
        node::{NodeElement, NodeProperty, SRef, UUID},
        parser::{
            blob, get_depth, node_content, prop_blob, prop_rbind, property, uuid, word, VecID,
        },
    };

    use super::node;

    #[test]
    fn uuid_test() {
        let res = uuid("#(1.2.3)");
        assert_eq!(res, Ok(("", vec![1, 2, 3])));
    }

    #[test]
    fn blob_test() {
        let res = blob("{blob}(1.2.3)");
        assert_eq!(res, Ok(("", (String::from("blob"), vec![1, 2, 3]))));
    }

    #[test]
    fn prop_blob_test() {
        let res = prop_blob("blob");
        assert_eq!(res, Ok(("", NodeProperty::Blob)));
    }

    #[test]
    fn rbind_test() {
        let res = prop_rbind("rbind[0,1,2]");
        assert_eq!(res, Ok(("", NodeProperty::Rbind(vec![0, 1, 2]))))
    }

    #[test]
    fn prop_test() {
        let res = property("<blob>");
        assert_eq!(res, Ok(("", NodeProperty::Blob)));
    }

    #[test]
    fn word_test() {
        let res = word("word ");
        assert_eq!(res, Ok((" ", "word")))
    }

    #[test]
    fn get_depth_test() {
        let res = get_depth("\n  -");
        assert_eq!(res, Ok(("\n  -", 2)))
    }

    #[test]
    fn node_contet_test() {
        let res = node_content("- ciao #(1.2.3) {eccomi}(1.2.3) sono io <blob> <rbind[0,1]>\n");
        assert_eq!(
            res,
            Ok((
                "",
                vec![
                    NodeElement::Word("ciao"),
                    NodeElement::TempRef(vec![1, 2, 3]),
                    NodeElement::TempBlob((String::from("eccomi"), vec![1, 2, 3])),
                    NodeElement::Word("sono"),
                    NodeElement::Word("io"),
                    NodeElement::Property(NodeProperty::Blob),
                    NodeElement::Property(NodeProperty::Rbind(vec![0, 1]))
                ]
            ))
        );
    }

    #[test]
    fn node_test() {
        let mut wanted_uuids = HashMap::<VecID, RefCell<Vec<Box<dyn Fn(SRef<UUID>) -> ()>>>>::new();
        let res = node(
            "- A \n - A.A #(0.1.0) {test}(0.0.1) \n  - A.A.A <rbind[0,1]> \n  - A.A.B \n - A.B \n  - A.B.A \n",
            None,
            &mut wanted_uuids,
        );

        let (out, res) = res.unwrap();

        print!("{}", out);

        println!("{}", res.borrow());

        assert_eq!(
            format!("{}", res.borrow()),
            "- (0) A \n - (0.0) A.A #(0.1.0) {test}(0.0.1) \n  - (0.0.0) A.A.A <rbind[0,1]> \n  - (0.0.1) A.A.B \n - (0.1) A.B \n  - (0.1.0) A.B.A \n"
        );
    }
}
