extern crate nom;

use super::node::NodeContent;

use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until, take_while, take_while_m_n},
    character::{
        complete::char, complete::digit1, is_alphabetic, is_alphanumeric, is_digit, is_space,
    },
    combinator::{map, map_res},
    multi::{fold_many0, many_till, separated_list1},
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

#[rustfmt::skip]
fn word(input: &str) -> IResult<&str, String> {
    preceded(
        tag(" "), 
        map(
            terminated(
                take_until(" "),tag(" ")
            ), 
            |word| String::from(word))
    )
    .parse(input)
}

#[rustfmt::skip]
fn node_content_fragment(input: &str) -> IResult<&str, NodeContent> {
    alt(
        (
            map(uuid, |ref_id| NodeContent::TempRef(ref_id)),
            map(word, |word| NodeContent::Text(word))
        )
    )
    .parse(input)
}

fn node_content(input: &str) -> IResult<&str, Vec<NodeContent>> {
    fold_many0(
        // Our parser function– parses a single string fragment
        node_content_fragment,
        // Our init value, an empty string
        Vec::new,
        // Our folding function. For each fragment, append the fragment to the
        // string
        |mut contents: Vec<NodeContent>, fragment| {
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
        node::NodeContent,
        parser::{uuid, node_content},
    };

    #[test]
    fn uuid_test() {
        let res = uuid("#(1.2.3)");
        assert_eq!(res, Ok(("", vec![1, 2, 3])));
    }

    #[test]
    fn node_ez_test() {
        let res = node_content(" ciao #(1.2.3) eccomi ");
        assert_eq!(
            res,
            Ok((
                "",
                vec![
                    NodeContent::Text(String::from("ciao")),
                    NodeContent::TempRef(vec![1, 2, 3]),
                    NodeContent::Text(String::from("eccomi"))
                ]
            ))
        );
    }
}
