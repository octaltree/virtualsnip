// https://github.com/Microsoft/language-server-protocol/blob/main/snippetSyntax.md
// https://github.com/microsoft/vscode/blob/main/src/vs/editor/contrib/snippet/snippetParser.ts
// NOTE: The implementation also has `${int/regex/format/options}`, but it is not documented.
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
    bytes::complete::{escaped, escaped_transform, tag, take_while1, take_while_m_n},
    character::complete::{anychar, char, digit1, none_of, one_of},
    combinator::{map, map_res, value},
    error::{Error, ErrorKind},
    multi::{many0, many1, separated_list1},
    sequence::delimited,
    Err, IResult
};
use std::{
    borrow::Cow,
    char::{decode_utf16, REPLACEMENT_CHARACTER},
    num::ParseIntError,
    u16
};

#[derive(Debug, PartialEq)]
pub struct Ast<'a>(Vec<Any<'a>>);

#[derive(Debug, PartialEq)]
pub enum Any<'a> {
    TabStop(TabStop),
    Placeholder(usize, Vec<Any<'a>>),
    Choice(usize, Vec<String>),
    Variable(&'a str, Box<C<'a>>),
    Text(&'a str)
}

#[derive(Debug, PartialEq)]
pub enum C<'a> {
    None,
    Any(Any<'a>),
    Transform(Regex<'a>, Options<'a>)
}

#[derive(Debug, PartialEq)]
pub struct Regex<'a>(&'a str);
#[derive(Debug, PartialEq)]
pub struct Options<'a>(&'a str);
#[derive(Debug, PartialEq)]
pub enum Format<'a> {
    Matched(usize),
    Upcase(usize),
    Downcase(usize),
    Capitalize(usize),
    If(usize, &'a str),
    IfElse(usize, &'a str, &'a str),
    Else(usize, &'a str),
    Text(&'a str)
}

pub type TabStop = usize;

pub fn parse(s: &str) -> Option<Ast<'_>> {
    let result = map(many0(any), Ast)(s);
    match result {
        Err(e) => {
            eprintln!("{}", e);
            None
        }
        Ok((rest, _)) if !rest.is_empty() => {
            eprintln!("trailing {}", rest);
            None
        }
        Ok((_, x)) => Some(x)
    }
}

fn any(s: &str) -> IResult<&str, Any<'_>> { alt((tab_stop_or_var, choice, placeholder))(s) }

/// $0 || ${0} || $var || ${var}
fn tab_stop_or_var(s: &str) -> IResult<&str, Any<'_>> {
    alt((
        map(tab_stop, Any::TabStop),
        map(var, |s| Any::Variable(s, Box::new(C::None)))
    ))(s)
}

/// $0 || ${0}
fn tab_stop(s: &str) -> IResult<&str, TabStop> {
    map_res(
        alt((
            map(permutation((char('$'), digit1)), |(_, b): (char, &str)| b),
            delimited(tag("${"), digit1, char('}'))
        )),
        |s| s.parse::<usize>()
    )(s)
}

/// $var || ${var}
fn var(s: &str) -> IResult<&str, &str> {
    alt((
        map(permutation((char('$'), var_name)), |(_, s): (char, &str)| s),
        map(
            permutation((tag("${"), var_name, char('}'))),
            |(_, s, _): (&str, &str, char)| s
        )
    ))(s)
}

/// ${0|text(,text)*|}
fn choice(s: &str) -> IResult<&str, Any<'_>> {
    // choice      ::= '${' int '|' text (',' text)* '|}'
    map(
        permutation((
            tag("${"),
            digit1,
            delimited(char('|'), choice_elements, char('|')),
            char('}')
        )),
        |(_, n, xs, _): (&str, &str, Vec<String>, char)| {
            let n = n.parse::<usize>().unwrap();
            Any::Choice(n, xs)
        }
    )(s)
}

fn choice_elements(s: &str) -> IResult<&str, Vec<String>> {
    separated_list1(
        char(','),
        escaped_transform(
            none_of(r#"\|,"#),
            '\\',
            alt((
                value('\\', char('\\')),
                value('|', char('|')),
                value(',', char(','))
            ))
        )
    )(s)
}

/// ${0:ast}
fn placeholder(s: &str) -> IResult<&str, Any<'_>> {
    let (rest, number) = map(permutation((tag("${"), digit1)), |(_, n): (&str, &str)| {
        n.parse::<usize>().unwrap()
    })(s)?;
    map(
        permutation((char(':'), many1(any), char('}'))),
        move |(_, children, _): (char, Vec<Any<'_>>, char)| Any::Placeholder(number, children)
    )(rest)
}

fn variable(s: &str) -> IResult<&str, Any<'_>> { todo!() }

fn var_name(s: &str) -> IResult<&str, &str> {
    let alphanum = take_while1(|c: char| c == '_' || c.is_ascii_alphanumeric())(s)?;
    // for first char
    let _ = take_while1(|c: char| c == '_' || c.is_ascii_alphabetic())(alphanum.1)?;
    Ok(alphanum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_tab_stop() {
        assert_eq!(tab_stop("$01a"), Ok(("a", 1)));
        assert_eq!(tab_stop("${0}"), Ok(("", 0)));
        assert_eq!(tab_stop("${00}a"), Ok(("a", 0)));
        assert!(tab_stop(" ${0}").is_err());
        assert!(tab_stop("${}").is_err());
        assert!(tab_stop("${-0}a").is_err());
    }

    #[test]
    fn can_var() {
        assert_eq!(var("$_a"), Ok(("", "_a")));
        assert_eq!(var("${_3}"), Ok(("", "_3")));
        assert_eq!(var("${a3}"), Ok(("", "a3")));
        assert!(var("${3}").is_err());
        assert!(var("$3").is_err());
    }

    #[test]
    fn can_choice() {
        let e = |r: IResult<&str, Any<'_>>| match r.unwrap().1 {
            Any::Choice(_, xs) => xs,
            _ => unreachable!()
        };
        assert_eq!(e(choice("${0|a|}")), &["a"]);
        assert_eq!(e(choice("${0|a,b|}")), &["a", "b"]);
        assert!(choice("${0|a,,b|}").is_err());
        assert_eq!(e(choice(r#"${0|\\a\,,b\||}"#)), &[r#"\a,"#, r#"b|"#]);
    }

    #[test]
    fn can_placeholder() {
        assert_eq!(
            placeholder("${30:${3:${2}}}").unwrap(),
            (
                "",
                Any::Placeholder(30, vec![Any::Placeholder(3, vec![Any::TabStop(2)])])
            )
        );
    }

    //#[test]
    // fn can_parse() {
    //    assert_eq!(
    //        parse("$_a$01a"),
    //        Some(Ast(vec![
    //            Any::Variable("_a", Box::new(C::None)),
    //            Any::TabStop(1),
    //            Any::Text("a")
    //        ]))
    //    );
    //}
}

//    DollarInt(usize),
//    DollarChar(char),
//    DollarBraces(&'a str),
//    Text(&'a str)
//}

// fn dollar_int(s: &str) -> IResult<&str, Any<'_>> {
//    map(permutation((char('$'), digit1)), |(_, s): (_, &str)| {
//        let n = s.parse::<usize>().unwrap();
//        Any::DollarInt(n)
//    })(s)
//}

// fn first_char(s: &str) -> IResult<&str, char> {
//    let o = s.chars().next();
//    match o {
//        Some(c) if c.is_ascii_alphabetic() || c == '_' => Ok((&s[1..], c)),
//        _ => Err(nom::Err::Failure(nom::error::Error::new(
//            s,
//            ErrorKind::Char
//        )))
//    }
//}

// fn var_name(s: &str) -> IResult<&str, &str> {
//    take_while1(|c: char| c == '_' || c.is_ascii_alphabetic())(s)
//}

// fn dollar_char(s: &str) -> IResult<&str, Any<'_>> {
//    map(permutation((char('$'), first_char)), |(_, c)| {
//        Any::DollarChar(c)
//    })(s)
//}

// fn dollar_braces(s: &str) -> IResult<&str, Any<'_>> {
//    let e = escaped(none_of("}"), '\\', one_of("}"));
//    map(delimited(tag("${"), e, char('}')), |s| Any::DollarBraces(s))(s)
//}

//#[derive(Debug, PartialEq, Eq)]
// struct TabStop(usize);

//#[derive(Debug)]
// struct Placeholder<'a>(usize, Box<Any<'a>>);

//#[derive(Debug, PartialEq)]
// struct Choice<'a>(usize, &'a str);

//#[derive(Debug, PartialEq)]
// enum Variable {}

//#[derive(Debug, PartialEq, Eq)]
// struct Var<'a>(&'a str);

//#[derive(Debug, PartialEq, Eq)]
// struct Text<'a>(Cow<'a, str>);

// impl<'a> From<&'a str> for Text<'a> {
//    fn from(s: &'a str) -> Self { Text(Cow::Borrowed(s)) }
//}

//// fn any(s: &str) -> IResult<&str, Any<'_>> { todo!() }

// fn tab_stop(s: &str) -> IResult<&str, TabStop> {
//    map_res(
//        alt((
//            map(permutation((char('$'), digit1)), |(_, b): (char, &str)| b),
//            delimited(tag("${"), digit1, char('}'))
//        )),
//        |s| -> Result<TabStop, ParseIntError> { Ok(TabStop(s.parse::<usize>()?)) }
//    )(s)
//}

// fn choice(s: &str) -> IResult<&str, Choice<'_>> {
//    fn parse_str<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
//        escaped(none_of("|"), '\\', one_of("\\|"))(i)
//    }
//    let mut p = map_res(
//        delimited(
//            tag("${"),
//            permutation((digit1, delimited(char('|'), parse_str, char('|')))),
//            char('}')
//        ),
//        |(n, s): (&str, &str)| -> Result<Choice<'_>, ParseIntError> {
//            Ok(Choice(n.parse::<usize>()?, s))
//        }
//    );
//    p(s)
//}

//#[cfg(test)]
// mod tests {
//    use super::*;

//    #[test]
//    fn can_first_char() {
//        assert_eq!(first_char("_"), Ok(("", '_')));
//        assert_eq!(first_char("_a"), Ok(("a", '_')));
//        assert!(first_char("あ").is_err());
//    }

//#[test]
// fn can_dollar_braces() {
//    assert_eq!(dollar_braces("${3}"), Ok(("", Any::DollarBraces("3"))));
//    assert_eq!(
//        dollar_braces("${3|a|}"),
//        Ok(("", Any::DollarBraces("3|a|")))
//    );
//    assert_eq!(
//        dollar_braces("${3|a\\}|}"),
//        Ok(("", Any::DollarBraces("3|a}|")))
//    );
//}

//#[test]
// fn can_tab_stop() {
//    assert_eq!(tab_stop("$01a"), Ok(("a", TabStop(1))));
//    assert_eq!(tab_stop("${0}"), Ok(("", TabStop(0))));
//    assert_eq!(tab_stop("${00}a"), Ok(("a", TabStop(0))));
//    assert!(tab_stop(" ${0}").is_err());
//    assert!(tab_stop("${}").is_err());
//    assert!(tab_stop("${-0}a").is_err());
//}

//#[test]
// fn can_choice() {
//    assert!(choice("${0}").is_err());
//    assert_eq!(choice(r#"${2|a,,b|}"#), Ok(("", Choice(2, "a,,b"))));
//    assert_eq!(choice(r#"${2|a,,b\||}"#), Ok(("", Choice(2, "a,,b|"))));
//    // assert_eq!(
//    //    choice(r#"${3|\a\\\,,,\|b|}"#),
//    //    Ok((
//    //        "",
//    //        Choice(3, vec![r#"\a\,"#.into(), "".into(), "|b".into()])
//    //    ))
//    //);
//}
