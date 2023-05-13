use std::{any::Any, rc::Rc, error::Error};

use nom::{bytes::complete::{tag, take, take_till, take_till1, is_a, take_while}, IResult, character::{complete::{char, digit1, anychar, alpha0, not_line_ending, one_of, alphanumeric0}, is_alphanumeric}, sequence::{preceded, separated_pair, delimited, tuple, pair, terminated}, multi::{length_count, many0}, combinator::{map_res, recognize, map}, branch::{alt, permutation}};
use tracing::{debug, info};

type CerealError = crate::util::CerealError;

#[derive(Debug, PartialEq, Eq)]
pub enum Parsed<'a> {
    Integer(&'a str),
    Double((&'a str, &'a str)),
    Null(&'a str),
    Object(&'a str),
    Array(Vec<Parsed<'a>>)
}


#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    Integer,
    Double,
    Array,
    Object,
    String,
    Null,
}

#[derive(Debug, Clone)]
pub struct Node<'a, T> {
    pub node_type: NodeType,
    pub value: Option<T>,
    pub children: Vec<Rc<Node<'a, T>>>,
    parent: Option<Rc<&'a Node<'a, T>>>,
}

pub fn parse_string<'a>(input: &str) -> IResult<&str, (&str, &str)> {
    //First step is to eat the 's:'
        pair(
            delimited(
                tag("s:"),
                digit1, 
                tag(":")), 
                delimited(
                    tag("\""), 
                    alphanumeric0, 
                    tag("\";")
                )
            )(input)

}
pub fn parse_double(input: &str) -> IResult<&str, Parsed<'_>> {
    delimited(
        tag("d:"),
        map(separated_pair(
            digit1, 
            tag("."), 
            digit1), |e| {Parsed::Double(e)}), 
        tag(";")
    )(input)
}
pub fn parse_int(input: &str) -> IResult<&str, Parsed<'_>> {
    map(delimited(tag("i:"), digit1, tag(";")), |e| {Parsed::Integer(e)})(input)
}

pub fn parse_null(input: &str) -> IResult<&str, Parsed<'_>> {
    map(tag("N;"), |e| Parsed::Null(e))(input)
}


pub fn parse_array(input: &str) -> IResult<&str, Parsed<'_>> {
    map(terminated(length_count(
        map(delimited(
            tag("a:"), 
            digit1, 
            tag(":{")
        ), |e: &str| -> usize {e.parse().unwrap()}), 
                alt((parse_int, parse_null, parse_double, parse_array)), 
    ), tag("}")), |e| Parsed::Array(e))(input)
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_valid_string() {
        assert_eq!(parse_string(r#"s:5:"value";"#), Ok(("", ("5", "value"))));
    }

    #[test]
    fn parse_valid_double() {
        assert_eq!(parse_double(r#"d:5.5;"#), Ok(("", Parsed::Double(("5", "5")))))
    }

    #[test]
    fn parse_valid_int() {
        assert_eq!(parse_int(r#"i:5;"#), Ok(("", Parsed::Integer("5"))))
    }

    #[test]
    fn parse_valid_null() {
        assert_eq!(parse_null(r#"N;"#), Ok(("", Parsed::Null("N;"))));

    }
    #[test]
    fn parse_simple_array() {
        let t = parse_array(r#"a:1:{i:5;}"#);
        assert_eq!(t, Ok(("", Parsed::Array(vec![Parsed::Integer("5")]))));
    }

    #[test]
    fn parse_nested_array() {
        let t = parse_array(r#"a:1:{a:1:{i:5;}}"#);
        assert_eq!(t, Ok(("", Parsed::Array(vec![Parsed::Array(vec![Parsed::Integer("5")])]))));
    }

    #[test]
    fn parse_empty_array() {
        let t = parse_array(r#"a:0:{}"#);
        assert_eq!(t, Ok(("", Parsed::Array(vec![]))))
    }
}