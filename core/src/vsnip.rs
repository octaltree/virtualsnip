// https://github.com/Microsoft/language-server-protocol/blob/main/snippetSyntax.md
// any         ::= tabstop | placeholder | choice | variable | text
// tabstop     ::= '$' int | '${' int '}'
// placeholder ::= '${' int ':' any '}'
// choice      ::= '${' int '|' text (',' text)* '|}'
// variable    ::= '$' var | '${' var }'
//                | '${' var ':' any '}'
//                | '${' var '/' regex '/' (format | text)+ '/' options '}'
// format      ::= '$' int | '${' int '}'
//                | '${' int ':' '/upcase' | '/downcase' | '/capitalize' '}'
//                | '${' int ':+' if '}'
//                | '${' int ':?' if ':' else '}'
//                | '${' int ':-' else '}' | '${' int ':' else '}'
// regex       ::= JavaScript Regular Expression value (ctor-string)
// options     ::= JavaScript Regular Expression option (ctor-options)
// var         ::= [_a-zA-Z] [_a-zA-Z0-9]*
// int         ::= [0-9]+
// text        ::= .*

use nom::{
    branch::{alt, permutation},
    bytes::complete::{escaped_transform, tag, take_while_m_n},
    character::complete::{anychar, char, digit1, none_of},
    combinator::{map, map_res, value},
    error::ErrorKind,
    multi::{many0, separated_list1},
    sequence::delimited,
    Err, IResult
};
use std::{
    char::{decode_utf16, REPLACEMENT_CHARACTER},
    num::ParseIntError,
    u16
};

pub fn parse(s: &str) { todo!() }

#[derive(Debug)]
struct Ast<'a>(Vec<Any<'a>>);

#[derive(Debug)]
enum Any<'a> {
    TabStop(TabStop),
    Placeholder(Placeholder<'a>),
    Choice(Choice<'a>),
    Variable(Variable),
    Text(Text<'a>)
}

#[derive(Debug, PartialEq, Eq)]
struct TabStop(usize);

#[derive(Debug)]
struct Placeholder<'a>(usize, Box<Any<'a>>);

#[derive(Debug, PartialEq)]
struct Choice<'a>(usize, Vec<Text<'a>>);

#[derive(Debug, PartialEq)]
enum Variable {}

#[derive(Debug, PartialEq, Eq)]
struct Var<'a>(&'a str);

#[derive(Debug, PartialEq, Eq)]
struct Text<'a>(&'a str);

// fn any(s: &str) -> IResult<&str, Any<'_>> { todo!() }

fn tab_stop(s: &str) -> IResult<&str, TabStop> {
    map_res(
        alt((
            map(permutation((char('$'), digit1)), |(_, b): (char, &str)| b),
            delimited(tag("${"), digit1, char('}'))
        )),
        |s| -> Result<TabStop, ParseIntError> { Ok(TabStop(s.parse::<usize>()?)) }
    )(s)
}

fn choice(s: &str) -> IResult<&str, Choice<'_>> {
    let mut p = map_res(
        delimited(tag("${"), permutation((digit1, choice_list)), char('}')),
        |(n, s): (&str, Vec<&str>)| -> Result<Choice<'_>, ParseIntError> {
            Ok(Choice(
                n.parse::<usize>()?,
                s.into_iter().map(Text).collect()
            ))
        }
    );
    p(s)
}

fn choice_list(s: &str) -> IResult<&str, Vec<&str>> {
    // TODO: escape
    let mut p = delimited(
        char('|'),
        |s: &str| -> IResult<&str, Vec<&str>> { todo!() },
        // separated_list1(char(','), escaped_transform(todo!(), '\\', todo!())),
        char('|')
    );
    p(s)
}

fn string_literal(s: &str) -> IResult<&str, String> {
    delimited(
        char('\"'),
        escaped_transform(
            none_of("\"\\"),
            '\\',
            alt((
                value('\\', char('\\')),
                value('\"', char('\"')),
                value('\'', char('\'')),
                value('\r', char('r')),
                value('\n', char('n')),
                value('\t', char('t')),
                map(
                    permutation((
                        char('u'),
                        take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit())
                    )),
                    |(_, code): (char, &str)| -> char {
                        decode_utf16(vec![u16::from_str_radix(code, 16).unwrap()])
                            .nth(0)
                            .unwrap()
                            .unwrap_or(REPLACEMENT_CHARACTER)
                    }
                )
            ))
        ),
        char('\"')
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_tab_stop() {
        assert_eq!(tab_stop("$01a"), Ok(("a", TabStop(1))));
        assert_eq!(tab_stop("${0}"), Ok(("", TabStop(0))));
        assert_eq!(tab_stop("${00}a"), Ok(("a", TabStop(0))));
        assert!(tab_stop(" ${0}").is_err());
        assert!(tab_stop("${}").is_err());
        assert!(tab_stop("${-0}a").is_err());
    }

    #[test]
    fn main() {
        assert_eq!(
            string_literal("\"a\\\"b\\\'c\""),
            Ok(("", String::from("a\"b\'c")))
        );
        assert_eq!(
            string_literal("\" \\r\\n\\t \\u2615 \\uDD1E\""),
            Ok(("", String::from(" \r\n\t ☕ �")))
        );
    }
}
