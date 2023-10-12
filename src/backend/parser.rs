extern crate nom;

use super::node::{NodeElement, NodeProperty};

use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until, take_while, take_while_m_n},
    character::{
        complete::char, complete::digit1, is_alphabetic, is_alphanumeric, is_digit, is_space,
    },
    combinator::{map, map_res},
    multi::{fold_many0, many_till, separated_list1, many0},
    sequence::{delimited, preceded, tuple, terminated},
    IResult, Parser,
};

type TempRefID = Vec<u32>;

pub fn parse_numbers(input: &str) -> IResult<&str, u32> {
    map_res(digit1, u32::from_str)(input)
}

fn uuid(input: &str) -> IResult<&str, TempRefID> {
    preceded(
        tag("#"),
        delimited(tag("("), separated_list1(tag("."), parse_numbers), tag(")")),
    )
    .parse(input)
}

fn blob(input: &str) -> IResult<&str, NodeProperty> {
    map(tag("blob"), |blob| NodeProperty::Blob).parse(input)
}

fn property(input: &str) -> IResult<&str, NodeProperty> {
    delimited(tag("<"),
    alt((blob,)),
    tag(">")
    ).parse(input)
}

#[rustfmt::skip]
fn word(input: &str) -> IResult<&str, &str> {
    preceded(
        many0(tag(" ")), 
        terminated(
                take_until(" "),tag(" ")
            )
    )
    .parse(input)
}

#[rustfmt::skip]
fn node_element(input: &str) -> IResult<&str, NodeElement> {
    alt(
        (
            map(uuid, |ref_id| NodeElement::TempRef(ref_id)),
            map(property, |property| NodeElement::Property(property)),
            map(word, |text| NodeElement::Word(text))
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
        parser::{uuid, node_content, blob, property, word},
    };

    #[test]
    fn uuid_test() {
        let res = uuid("#(1.2.3)");
        assert_eq!(res, Ok(("", vec![1, 2, 3])));
    }

    #[test]
    fn blob_test() {
        let res = blob("blob");
        assert_eq!(res, Ok(("", NodeProperty::Blob)));
    }

    #[test]
    fn prop_test() {
        let res = property("<blob>");
        assert_eq!(res, Ok(("", NodeProperty::Blob)));
    }

    #[test]
    fn word_test() {
        let res = word(" word ");
        assert_eq!(res, Ok(("", "word")))
    }

    #[test]
    fn node_contet_test() {
        let res = node_content(" ciao #(1.2.3) eccomi sono io <blob>");
        assert_eq!(
            res,
            Ok((
                "",
                vec![
                    NodeElement::Word("ciao"),
                    NodeElement::TempRef(vec![1, 2, 3]),
                    NodeElement::Word("eccomi"),
                    NodeElement::Word("sono"),
                    NodeElement::Word("io"),
                    NodeElement::Property(NodeProperty::Blob)
                ]
            ))
        );
    }
}
