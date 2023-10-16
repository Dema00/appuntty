extern crate nom;

use crate::backend::node::NodeContent;

use super::node::{NodeElement, NodeProperty, Node, UUID};

use core::panic;
use std::{str::FromStr, pin::Pin};

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until, take_while, take_while_m_n, is_a},
    character::{
        complete::char, complete::digit1, is_alphabetic, is_alphanumeric, is_digit, is_space,
    },
    combinator::{map, map_res, value},
    multi::{fold_many0, many_till, separated_list1, many0},
    sequence::{delimited, preceded, tuple, terminated, Tuple, pair},
    IResult, Parser,
};

//Data Structures



type TempRefID = Vec<u32>;

//Text Parser Elements

fn parse_numbers(input: &str) -> IResult<&str, u32> {
    map_res(digit1, u32::from_str)(input)
}

fn parse_id(input: &str) -> IResult<&str, TempRefID> {
    delimited(tag("("), separated_list1(tag("."), parse_numbers), tag(")")).parse(input)
}

fn uuid(input: &str) -> IResult<&str, TempRefID> {
    preceded(
        tag("#"),
        parse_id,
    )
    .parse(input)
}

fn blob(input: &str) -> IResult<&str, (String, Vec<u32>)> {
    pair(
    delimited(tag("{"), map(take_until("}"),|str| String::from(str)), tag("}")),
    parse_id).parse(input)
}

fn prop_blob(input: &str) -> IResult<&str, NodeProperty> {
    map(tag("blob"), |blob| NodeProperty::Blob).parse(input)
}

fn prop_rbind(input: &str) -> IResult<&str, NodeProperty> {
    preceded(
        tag("rbind"),
        delimited(
            tag("["),
            map(
                separated_list1(tag(","), parse_numbers),
                |vec| NodeProperty::Rbind(vec)),
            tag("]"))
    ).parse(input)
}

fn property(input: &str) -> IResult<&str, NodeProperty> {
    delimited(tag("<"),
    alt((prop_blob, prop_rbind)),
    tag(">")
    ).parse(input)
}

#[rustfmt::skip]
fn word(input: &str) -> IResult<&str, &str> {
    take_until(" ").parse(input)
}

#[rustfmt::skip]
fn node_element(input: &str) -> IResult<&str, NodeElement> {
    preceded(tag(" "),
        alt(
            (
                map(uuid, |ref_id| NodeElement::TempRef(ref_id)),
                map(property, |property| NodeElement::Property(property)),
                map(blob, |blob| NodeElement::TempBlob(blob)),
                map(word, |text| NodeElement::Word(text))
            )
        )
    )
    .parse(input)
}

fn node_content(input: &str) -> IResult<&str, Vec<NodeElement>> {
    fold_many0(
        node_element,
        Vec::new,
        |mut contents: Vec<NodeElement>, fragment| {
            contents.push(fragment);
            contents
        },
    )
    .parse(input)
}

fn node<'p, 'i>(parent: &'p Node ,input: &'i str) -> IResult<&'i str, Node<'p> >{
    preceded(
        take_until("-"),
        map(node_content, |elements| build_node(parent, elements))
    ).parse(input)
}

fn build_node<'p>(parent: &'p Node, elements: Vec<NodeElement>) -> Node<'p> {
    elements.into_iter().fold( Node::new(parent), |mut new_node, el|
        match el {
            NodeElement::Word(text) => {
                new_node.cont.push(NodeContent::Text(String::from(text)));
                new_node
            },
            NodeElement::TempRef(addr_vec) => {
                new_node.cont.push(NodeContent::Reference(get_uuid(parent.root(), &addr_vec)));
                new_node
            },
            NodeElement::TempBlob((text,addr_vec)) => {
                new_node.cont.push(NodeContent::Blob((text, get_uuid(parent.root(), &addr_vec))));
                new_node
            },
            NodeElement::Property(property) => {
                new_node.prop.push(property);
                new_node
            }
        }
    );
    todo!()
}

fn get_uuid<'r, 'p>(root: &'p Node<'r>, addr_vec: &Vec<u32>) -> UUID<'p> {
    addr_vec.first().map_or( root.uuid, |idx| 
        get_uuid(root.sons
            .first()
            .unwrap_or(panic!("Parsing error: specified UUID not found")), addr_vec)
    )
}

/*fn next_node<'t, 'p, P: NodeProperty>(
    i: &'t [u8],
    parent_uuid: &'p UUID,
    status: ParserStatus,
) -> IResult<&'t [u8], Node<'p, P>> {
    todo!()
}
*/

#[cfg(test)]
mod tests {
    use crate::backend::{
        node::{NodeContent, NodeElement, NodeProperty},
        parser::{uuid, node_content, prop_blob, property, word, prop_rbind, blob},
    };

    #[test]
    fn uuid_test() {
        let res = uuid("#(1.2.3)");
        assert_eq!(res, Ok(("", vec![1, 2, 3])));
    }

    #[test]
    fn blob_test() {
        let res = blob("{blob}(1.2.3)");
        assert_eq!(res, Ok(("", (String::from("blob"), vec![1,2,3]))));
    }

    #[test]
    fn prop_blob_test() {
        let res = prop_blob("blob");
        assert_eq!(res, Ok(("", NodeProperty::Blob)));
    }

    #[test]
    fn rbind_test() {
        let res = prop_rbind("rbind[0,1,2]");
        assert_eq!(res, Ok(("",NodeProperty::Rbind(vec![0,1,2]))))
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
    fn node_contet_test() {
        let res = node_content(" ciao #(1.2.3) {eccomi}(1.2.3) sono io <blob> <rbind[0,1]>");
        assert_eq!(
            res,
            Ok((
                "",
                vec![
                    NodeElement::Word("ciao"),
                    NodeElement::TempRef(vec![1, 2, 3]),
                    NodeElement::TempBlob((String::from("eccomi"),vec![1,2,3])),
                    NodeElement::Word("sono"),
                    NodeElement::Word("io"),
                    NodeElement::Property(NodeProperty::Blob),
                    NodeElement::Property(NodeProperty::Rbind(vec![0,1]))
                ]
            ))
        );
    }
}
