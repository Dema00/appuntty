extern crate nom;

use crate::backend::node::NodeContent;

use super::node::{HRef, Node, NodeElement, NodeProperty, SRef, WHRef, WSRef, UUID};

use core::panic;
use std::{
    pin::Pin,
    rc::{Rc, Weak},
    str::FromStr,
};

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take, take_till, take_until, take_while, take_while_m_n},
    character::{
        complete::char,
        complete::{digit1, multispace0, multispace1, newline, one_of},
        is_alphabetic, is_alphanumeric, is_digit, is_space,
    },
    combinator::{map, map_res, peek, value},
    error::{Error, ParseError},
    multi::{count, fold_many0, many0, many0_count, many_till, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple, Tuple},
    IResult, Parser,
};

//Data Structures

type TempRefID = Vec<usize>;

//Text Parser Elements

fn parse_numbers(input: &str) -> IResult<&str, usize> {
    map_res(digit1, usize::from_str).parse(input)
}

fn parse_id(input: &str) -> IResult<&str, TempRefID> {
    delimited(tag("("), separated_list1(tag("."), parse_numbers), tag(")")).parse(input)
}

fn uuid(input: &str) -> IResult<&str, TempRefID> {
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
    map(tag("blob"), |blob| NodeProperty::Blob).parse(input)
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
/*node_element,
Vec::new,
|mut contents: Vec<NodeElement>, fragment| {
    contents.push(fragment);
    println!("{:?}",contents);
    contents
},*/
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

fn node(input: &str, parent: HRef<Node>) -> IResult<&str, HRef<Node>> {
    let (_, depth) = get_depth.parse(input)?;

    let (mut input, contents) =
        preceded(preceded(many0(newline), take_until("-")), node_content).parse(input)?;

    let new_node = Node::new(Some(Rc::clone(&parent)));
    populate_node(Rc::clone(&new_node), contents);

    if !input.is_empty() {
        let (_, mut next_depth) = get_depth.parse(input)?;

        while depth < next_depth && !input.is_empty() {
            let child_node;
            (input, child_node) = node(input, Rc::clone(&new_node))?;
            new_node.borrow_mut().push_child(child_node);

            if !input.is_empty() {
                (_, next_depth) = get_depth.parse(input)?;
            }
        }
    }

    IResult::Ok((input, new_node))
}

fn populate_node(node: HRef<Node>, contents: Vec<NodeElement>) {
    let node = node.borrow_mut();
    for element in contents {
        match element {
            NodeElement::Word(word) => node.push_content(NodeContent::Text(String::from(word))),
            NodeElement::Property(property) => node.push_property(property),
            NodeElement::TempBlob((word, addr)) => {
                node.push_content(NodeContent::Blob((word, get_uuid(addr))))
            }
            NodeElement::TempRef(addr) => node.push_content(NodeContent::Reference(get_uuid(addr))),
        }
    }
}

fn get_uuid(addr_vec: TempRefID) -> WSRef<UUID> {
    let uuid = UUID::new(None, addr_vec[0]);
    Rc::downgrade(&Rc::new(uuid))
}

fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(f: F) -> impl Parser<&'a str, O, E> {
    delimited(multispace0, f, multispace0)
}

#[cfg(test)]
mod tests {
    use crate::backend::{
        node::{HRef, Node, NodeContent, NodeElement, NodeProperty, UUID},
        parser::{blob, get_depth, node_content, prop_blob, prop_rbind, property, uuid, word},
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
        let root = Node::new(None);
        let res = node(
            "- A \n - A.A \n  - A.A.A \n  - A.A.B \n - A.B \n  - A.B.A \n",
            root,
        );

        println!("{}", res.unwrap().1.borrow());

        assert!(false);
    }
}
