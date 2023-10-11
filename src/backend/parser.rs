extern crate nom;

use super::node::{NodeContent};

use std::str::FromStr;

use nom::{
    bytes::complete::{tag, take, take_while, take_while_m_n},
    character::{is_alphabetic, is_alphanumeric, is_digit, complete::digit1},
    combinator::map_res,
    sequence::{delimited, tuple},
    IResult, multi::{separated_list1, many_till}, branch::alt,
};

struct Parser {
    curr_id: u32,
}

type TempRefID = Vec<u32>;
type Word = Vec<String>;

pub fn parse_numbers(input: &str) -> IResult<&str, u32> {
    map_res(digit1, u32::from_str)(input)
}

fn uuid(input: &str) -> IResult<&str, TempRefID> {
    let (input, _) = tag("#")(input)?;
    let (input, temp_ref_id) = delimited(tag("("), separated_list1(tag("."),parse_numbers), tag(")"))(input)?;
    Ok((input, temp_ref_id))
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
    use crate::backend::parser::uuid;

    #[test]
    fn uuid_test() {
        let res = uuid("#(1.2.3)");
        assert_eq!(res, Ok(("",vec![1,2,3])));
    }
}
