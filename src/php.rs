use std::{any::Any, error::Error, rc::Rc, fmt::Display};


use nom::{
    branch::{alt, permutation},
    bytes::complete::{is_a, tag, take, take_till, take_till1, take_while},
    character::{
        complete::{
            alpha0, alphanumeric0, alphanumeric1, anychar, char, digit1, not_line_ending, one_of,
        },
        is_alphanumeric,
    },
    combinator::{map, map_res, recognize, eof},
    multi::{length_count, many0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use tracing::{debug, info};

type CerealError = crate::util::CerealError;

#[derive(Debug, PartialEq, Eq)]
pub enum Parsed<'a> {
    Integer(&'a str),
    Double((&'a str, &'a str)),
    Null(&'a str),
    Object((&'a str, &'a str, Vec<Parsed<'a>>)),
    ObjectVal(Box<(Parsed<'a>, Parsed<'a>)>),
    Bool(&'a str),
    Array(Vec<Parsed<'a>>),
    String(Vec<&'a str>),
    Eof,
}

impl Display for Parsed<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug!("displaying a parsed item");
        match self {
            Parsed::Integer(val) => write!(f, "{}", val),
            Parsed::Double((leading, decimal)) => write!(f, "{}.{}", leading, decimal),
            Parsed::Null(_) => write!(f, "Null"),
            Parsed::Object((length_of_name, name, items)) => write!(f, "({}, {}):({:?})", length_of_name, name, items),
            Parsed::ObjectVal(val) => write!(f, "{}: {}", val.0, val.1),
            Parsed::Bool(val) => write!(f, "{}", {
                match *val {
                    "0" => false,
                    "1" => true,
                    _ => unreachable!()
                }
            }),
            Parsed::Array(items) => write!(f, "[{}]", {
                let mut out: Vec<String> = Vec::new();
                // Only add a begining newline to arrays with items
                if (items.len() != 0) {out.push("\n".to_owned())}
                for item in items {
                    out.push(format!("{}, \n", item));
                }
                out.join("")
            }),
            Parsed::String(chars) => write!(f, "\"{}\"", chars.join("")),
            Parsed::Eof => write!(f, "EOF"),
        }
    }
}


pub fn parse_string<'a>(input: &str) -> IResult<&str, Parsed<'_>> {
    // s:5:"value";
    map(
        terminated(
            length_count(
                map(delimited(tag("s:"), digit1, tag(":")), |e: &str| -> usize {
                    debug!("Parsed string of length {}", &e);
                    e.parse::<usize>().unwrap() + 2
                }),
                take(1usize),
            ),
            tag(";"),
        ),
        // Trim the "" around the string as we dont need that and there isnt an easy way to drop it during parsing
        |e| {
            debug!(
                "Parsed string value '{}'",
                e[1usize..e.len() - 1 as usize].to_vec().join("")
            );
            Parsed::String(e[1usize..e.len() - 1 as usize].to_vec())
        },
    )(input)
}
pub fn parse_double(input: &str) -> IResult<&str, Parsed<'_>> {
    delimited(
        tag("d:"),
        map(separated_pair(digit1, tag("."), digit1), |e| {
            debug!("Parsed double {}.{}", &e.0, &e.1);
            Parsed::Double(e)
        }),
        tag(";"),
    )(input)
}

pub fn parse_bool(input: &str) -> IResult<&str, Parsed<'_>> {
    map(
        delimited(tag("b:"), alt((tag("0"), tag("1"))), tag(";")),
        |e| {
            debug!("Parsed bool {}", &e);
            Parsed::Bool(e)
        },
    )(input)
}

pub fn parse_int(input: &str) -> IResult<&str, Parsed<'_>> {
    map(delimited(tag("i:"), digit1, tag(";")), |e| {
        debug!("Parsed integer {}", &e);
        Parsed::Integer(e)
    })(input)
}

pub fn parse_null(input: &str) -> IResult<&str, Parsed<'_>> {
    map(tag("N;"), |e| {
        debug!("Parsed Null");
        Parsed::Null(e)
    })(input)
}

pub fn parse_eof(input: &str) -> IResult<&str, Parsed<'_>> {
    map(eof, |e| {
        debug!("reached end of input");
        Parsed::Eof
    })(input)
}

pub fn parse_any(input: &str) -> IResult<&str, Parsed<'_>> {
    alt((
        parse_int,
        parse_null,
        parse_double,
        parse_array,
        parse_object,
        parse_string,
        parse_bool,
        parse_eof,
    ))(input)
}

pub fn parse_array(input: &str) -> IResult<&str, Parsed<'_>> {
    map(
        terminated(
            length_count(
                map(
                    delimited(tag("a:"), digit1, tag(":{")),
                    |e: &str| -> usize { e.parse().unwrap() },
                ),
                // An array can literally be an array of entries, or it can have named entries like an object for some reason, lets just not pay any attention to php, mmkay?
                alt((map(tuple((parse_string, parse_any)), |e| {
                    debug!("Parsed object value {:?}", &e);
                    Parsed::ObjectVal(Box::new(e))
                }), parse_any)),
            ),
            tag("}"),
        ),
        |e| {
            debug!("Parsed array {:?}", &e);
            Parsed::Array(e)
        },
    )(input)
}


pub fn parse_object(input: &str) -> IResult<&str, Parsed<'_>> {
    map(
        // O:5:"Fruit":2:{s:4:"name";N;s:5:"color";N;}
        tuple((
            delimited(tag("O:"), digit1, tag(":")),
            terminated(delimited(tag("\""), alphanumeric1, tag("\"")), tag(":")),
            terminated(
                length_count(
                    map(terminated(digit1, tag(":{")), |e: &str| -> usize {
                        e.parse().unwrap()
                    }),
                    map(tuple((parse_string, parse_any)), |e| {
                        debug!("Parsed object value {:?}", &e);
                        Parsed::ObjectVal(Box::new(e))
                    }),
                ),
                tag("}"),
            ),
        )),
        |e| Parsed::Object(e),
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_valid_string() {
        assert_eq!(
            parse_string(r#"s:5:"value";"#),
            Ok(("", Parsed::String(vec!["v", "a", "l", "u", "e"])))
        );
    }

    #[test]
    fn parse_invalid_length_string() {
        assert!(parse_string(r#"s:2:"123";"#).is_err())
    }

    #[test]
    fn parse_empty_string() {
        assert_eq!(parse_string(r#"s:0:"";"#), Ok(("", Parsed::String(vec![]))));
    }

    #[test]
    fn parse_no_input() {
        assert!(parse_any("").is_ok());
        assert_eq!(parse_any("").unwrap().1, Parsed::Eof);
    }

    #[test]
    fn parse_valid_bool() {
        assert_eq!(parse_bool(r#"b:0;"#), Ok(("", Parsed::Bool("0"))));
    }

    #[test]
    fn parse_valid_double() {
        assert_eq!(
            parse_double(r#"d:5.5;"#),
            Ok(("", Parsed::Double(("5", "5"))))
        )
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
    fn parse_array_with_mutliple() {
        let t = parse_array(r#"a:2:{i:5;d:5.12312341241231231234;}"#);
        assert_eq!(
            t,
            Ok((
                "",
                Parsed::Array(vec![
                    Parsed::Integer("5"),
                    Parsed::Double(("5", "12312341241231231234"))
                ])
            ))
        );
    }

    #[test]
    fn parse_nested_array() {
        let t = parse_array(r#"a:1:{a:1:{i:5;}}"#);
        assert_eq!(
            t,
            Ok((
                "",
                Parsed::Array(vec![Parsed::Array(vec![Parsed::Integer("5")])])
            ))
        );
    }

    #[test]
    fn parse_empty_array() {
        let t = parse_array(r#"a:0:{}"#);
        assert_eq!(t, Ok(("", Parsed::Array(vec![]))))
    }

    #[test]
    fn set_of_lines() {
        let mut i: usize = 0;
        for line in include_str!("input.txt").split("\n") {
            debug!("Parsing line {i}: {}", &line);
            parse_any(line).unwrap();
            i += 1;
        }
    }

    #[test]
    fn parse_simple_object() {
        let t = parse_object(r#"O:5:"Fruit":2:{s:4:"name";N;s:5:"color";N;}"#);
        assert_eq!(
            t,
            Ok((
                "",
                Parsed::Object((
                    "5",
                    "Fruit",
                    vec![
                        Parsed::ObjectVal(Box::new((
                            Parsed::String(vec!["n", "a", "m", "e"]),
                            Parsed::Null("N;")
                        ))),
                        Parsed::ObjectVal(Box::new((
                            Parsed::String(vec!["c", "o", "l", "o", "r"]),
                            Parsed::Null("N;")
                        ))),
                    ]
                ))
            ))
        );
    }
}
