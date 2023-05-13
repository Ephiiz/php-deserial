use std::any::Any;

use nom::{bytes::complete::{tag, take, take_till, take_till1, is_a, take_while}, IResult, character::{complete::{char, digit1, anychar, alpha0, not_line_ending, one_of, alphanumeric0}, is_alphanumeric}, sequence::{preceded, separated_pair, delimited, tuple}, multi::length_count, combinator::map_res};
use tracing::{debug, info};

type CerealError = crate::util::CerealError;

#[derive(Debug)]
enum PhpObjectType {
    Integer,
    Double,
    String,
    Array,
    Object,
    Null,
}


#[derive(Debug)]
struct PhpObject<Inner> {
    pub obj_type: PhpObjectType,
    pub length: Option<usize>,
    pub value: Option<Inner>,
}

fn eat_colon(input: &str) -> IResult<&str, char> {
    char(':')(input)
}
fn eat_semicolon(input: &str) -> IResult<&str, char> {
    char(';')(input)
}

fn is_colon(input: char) -> bool {
    if input == ':' { true } else { false }
}

fn lex_int(input: &str) -> IResult<&str, (&str, i32, &str)> {
    tuple((
        tag(":"),
        nom::character::complete::i32,
        tag(";")
    ))(input)
}
fn parse_int(input: &str) -> Result<(&str, PhpObject<i32>), CerealError> {
    let (rem, (_, val, _)) = lex_int(input).expect("failed to lex integer object");
    Ok((rem,
        PhpObject {
            obj_type: PhpObjectType::Integer,
            length: None,
            value: Some(val),
        }))

}

fn lex_double(input: &str) -> IResult<&str, (&str, (i32, u32), &str)> {
    tuple((
        tag(":"),
        separated_pair(nom::character::complete::i32, tag("."), nom::character::complete::u32),
        tag(";"),
    ))(input)
}
fn parse_double(input: &str) -> Result<(&str, PhpObject<f32>), CerealError> {
    let (rem, (_, (first, second), _)) = lex_double(input).expect("failed to lex double object");
    let val: f32 = format!("{first}.{second}").parse().expect("failed to parse double into float");
    Ok((rem,
        PhpObject {
            obj_type: PhpObjectType::Double,
            length: None,
            value: Some(val),
        }))
}


/// Takes the input of a string object and lexes it to a tuple
fn lex_string(input: &str) -> IResult<&str, (&str, i32, &str, &str)> {
    tuple((
        tag(":"), 
        nom::character::complete::i32,
        tag(":"), 
        delimited(
            tag("\""), 
            alphanumeric0, 
            tag("\";")
            )
        )
    )(input)

}

/// Takes the tuple from `lex_string` and finishes parsing into a `PhpObject<&str>`
fn parse_string<'a>(input: &'a str) -> Result<(&'a str, PhpObject<&'a str>), CerealError> {
    // First and third &str are just the colons so we can ignore those
    let (rem, (_, length, _, value)) = lex_string(input).expect("failed to lex string object");
    Ok((rem, 
        PhpObject {
            obj_type: PhpObjectType::String,
            length: Some(length as usize),
            value: Some(value),
    }))
}

fn till_delim(input: &str, delim: char) -> IResult<&str, &str> {
    take_till(|c| c == delim)(input)
}

pub fn serialize(input: &'static str) -> Option<(&str, Box<dyn Any>)> {
    debug!("Starting serializing input: {:?}", &input);
    // First step in the serialization process is to determine what kind of object we are serializing
    if let Ok((remain, entry_type)) = till_delim(input, ':') {
        // There should only be a single character here anyways, if not then we have bad input and make no promises
        let entry_type = match entry_type.chars().nth(0) {
            Some('N') => Some(PhpObjectType::Null),
            Some('s') => Some(PhpObjectType::String),
            Some('a') => Some(PhpObjectType::Array),
            Some('O') => Some(PhpObjectType::Object),
            Some('d') => Some(PhpObjectType::Double),
            Some('i') => Some(PhpObjectType::Integer),
            Some(_) => todo!("flesh out incorrect object type handling"),
            None => None,
        };
        debug!("found entry type: {:?}", &entry_type);
        if entry_type.is_none() {
            return None;
        }

        // Unneeded branch here, but just for testing rn
        match entry_type.unwrap() {
            PhpObjectType::Integer => {
                let parsed = parse_int(remain).expect("failed to parse integer");
                info!("{:?}", parsed);
                return Some((parsed.0, Box::new(parsed.1)));
            },
            PhpObjectType::Double => {
                let parsed = parse_double(remain).expect("failed to parse double");
                info!("{:?}", parsed);
                return Some((parsed.0, Box::new(parsed.1)));
            },
            PhpObjectType::String => {
                let parsed = parse_string(remain).expect("failed to parse string");
                info!("{:?}", parsed);
                return Some((parsed.0, Box::new(parsed.1)));
            },

            PhpObjectType::Array => todo!(),
            PhpObjectType::Object => todo!(),
            PhpObjectType::Null => {
                let parsed = eat_semicolon(remain).expect("null object missing semicolon");

                //FIXME: we have to have some boxed value but this is null, so a blank string is prob no the best option
                return Some((parsed.0, Box::new(
                    PhpObject { obj_type: PhpObjectType::Null, length: None, value: Some(0) }
                )));
            },
        }
    } else {
        panic!("serialization failed to find delimiter ':'")
    }
}