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
    bytes::complete::{escaped, escaped_transform, tag, take_while1},
    character::complete::{char, digit1, none_of, one_of},
    combinator::{map, map_res, value},
    error::ErrorKind,
    multi::{many0, many1, separated_list1},
    sequence::delimited,
    IResult
};

#[derive(Debug, PartialEq)]
pub struct Ast<'a>(pub Vec<Any<'a>>);

#[derive(Debug, PartialEq)]
pub enum Any<'a> {
    TabStop(TabStop),
    Placeholder(usize, Vec<Any<'a>>),
    Choice(usize, Vec<String>),
    Variable(&'a str, V<'a>),
    Text(String)
}

#[derive(Debug, PartialEq)]
pub enum V<'a> {
    None,
    Any(Vec<Any<'a>>),
    Transform(Regex<'a>, Vec<Format<'a>>, Options<'a>)
}

/// escaped / and \self
#[derive(Debug, PartialEq)]
pub struct Regex<'a>(Escaped<'a>);
/// valid options has no } ?
#[derive(Debug, PartialEq)]
pub struct Options<'a>(&'a str);
#[derive(Debug, PartialEq, Clone)]
pub enum Format<'a> {
    Matched(usize),
    Upcase(usize),
    Downcase(usize),
    Capitalize(usize),
    If(usize, Escaped<'a>),
    IfElse(usize, Escaped<'a>, Escaped<'a>),
    Else(usize, Escaped<'a>),
    Text(&'a str)
}

/// unescaping needs reallocating string
pub type Escaped<'a> = &'a str;
pub type TabStop = usize;

pub fn parse(s: &str) -> Option<Ast<'_>> {
    // NOTE: if the parser passed to many0 accepts empty inputs (like alpha0 or digit0), many0 will return an error, to prevent going into an infinite loop
    let result = map(many0(any), Ast)(s);
    match result {
        Err(e) => {
            println!("{}", e);
            None
        }
        Ok((rest, _)) if !rest.is_empty() => {
            println!("trailing {}", rest);
            None
        }
        Ok((_, x)) => Some(x)
    }
}

fn any(s: &str) -> IResult<&str, Any<'_>> {
    if s.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(s, ErrorKind::Eof)));
    }
    alt((tab_stop_or_var_name, choice, placeholder, variable, text))(s)
}

fn any_inner_braces(s: &str) -> IResult<&str, Any<'_>> {
    if s.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(s, ErrorKind::Eof)));
    }
    alt((
        tab_stop_or_var_name,
        choice,
        placeholder,
        variable,
        text_inner_braces
    ))(s)
}

/// $0 || ${0} || $var || ${var}
fn tab_stop_or_var_name(s: &str) -> IResult<&str, Any<'_>> {
    alt((
        map(number, Any::TabStop),
        map(var, |s| Any::Variable(s, V::None))
    ))(s)
}

/// $0 || ${0}
fn number(s: &str) -> IResult<&str, usize> {
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

fn var_name(s: &str) -> IResult<&str, &str> {
    let alphanum = take_while1(|c: char| c == '_' || c.is_ascii_alphanumeric())(s)?;
    // for first char
    let _ = take_while1(|c: char| c == '_' || c.is_ascii_alphabetic())(alphanum.1)?;
    Ok(alphanum)
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
        permutation((char(':'), many1(any_inner_braces), char('}'))),
        move |(_, children, _): (char, Vec<Any<'_>>, char)| Any::Placeholder(number, children)
    )(rest)
}

///  ${' var ':' any '}' || '${' var '/' regex '/' (format | text)+ '/' options '}'
fn variable(s: &str) -> IResult<&str, Any<'_>> {
    let (rest, name) = map(
        permutation((tag("${"), var_name)),
        |(_, name): (&str, &str)| name
    )(s)?;
    let a = map(
        permutation((char(':'), many1(any_inner_braces), char('}'))),
        move |(_, children, _): (char, Vec<Any<'_>>, char)| {
            Any::Variable(<&str>::clone(&name), V::Any(children))
        }
    );
    let t = map(transform, move |v| Any::Variable(name, v));
    alt((a, t))(rest)
}

fn transform(s: &str) -> IResult<&str, V> {
    map(
        permutation((
            char('/'),
            regex,
            char('/'),
            formats,
            char('/'),
            options,
            char('}')
        )),
        |(_, r, _, f, _, o, _)| V::Transform(r, f, o)
    )(s)
}

fn regex(s: &str) -> IResult<&str, Regex<'_>> {
    map(escaped(none_of("\\/"), '\\', one_of(r#"/\"#)), Regex)(s)
}

fn options(s: &str) -> IResult<&str, Options<'_>> {
    match s.chars().next() {
        None => return Ok(("", Options(""))),
        Some('}') => return Ok((s, Options(""))),
        _ => ()
    }
    map(take_while1(|c| c != '}'), Options)(s)
}

fn formats(s: &str) -> IResult<&str, Vec<Format<'_>>> {
    // NOTE: if the parser passed to many0 accepts empty inputs (like alpha0 or digit0), many0 will return an error, to prevent going into an infinite loop
    many0(format)(s)
}

/// format      ::= '$' int | '${' int '}'
///                | '${' int ':' '/upcase' | '/downcase' | '/capitalize' '}'
///                | '${' int ':+' if '}'
///                | '${' int ':?' if ':' else '}'
///                | '${' int ':-' else '}' | '${' int ':' else '}'
fn format(s: &str) -> IResult<&str, Format<'_>> {
    if let Ok(t) = map(number, Format::Matched)(s) {
        return Ok(t);
    }
    fn matched_with_transform(s: &str) -> IResult<&str, Format<'_>> {
        let (s, n) = map(
            permutation((tag("${"), digit1, char(':'))),
            |(_, n, _): (&str, &str, char)| n.parse::<usize>().unwrap()
        )(s)?;
        let case = alt((
            value(Format::Upcase(n), tag("/upcase")),
            value(Format::Downcase(n), tag("/downcase")),
            value(Format::Capitalize(n), tag("/capitalize"))
        ));
        let i = map(
            permutation((char('+'), take_while1(|c| c != '}'))),
            move |(_, i): (_, &str)| Format::If(n, i)
        );
        let e = map(
            permutation((char('-'), take_while1(|c| c != '}'))),
            move |(_, i): (_, &str)| Format::Else(n, i)
        );
        let ie = map(
            permutation((
                char('?'),
                take_while1(|c| c != ':'),
                char(':'),
                take_while1(|c| c != '}')
            )),
            move |(_, i, _, e): (_, &str, _, &str)| Format::IfElse(n, i, e)
        );
        let e2 = map(take_while1(|c| c != '}'), move |e| Format::Else(n, e));
        map(
            permutation((alt((case, i, e, ie, e2)), char('}'))),
            |(f, _)| f
        )(s)
    }
    let t = map(take_while1(|c| c != '/'), Format::Text);
    alt((matched_with_transform, t))(s)
}

fn text(s: &str) -> IResult<&str, Any<'_>> {
    map(
        escaped_transform(
            none_of(r#"\$"#),
            '\\',
            alt((value('\\', char('\\')), value('$', char('$'))))
        ),
        Any::Text
    )(s)
}

fn text_inner_braces(s: &str) -> IResult<&str, Any<'_>> {
    map(
        escaped_transform(
            none_of(r#"\$}"#),
            '\\',
            alt((
                value('\\', char('\\')),
                value('$', char('$')),
                value('}', char('}'))
            ))
        ),
        Any::Text
    )(s)
}

// TODO: test with https://github.com/microsoft/vscode/blob/main/src/vs/editor/contrib/snippet/test/snippetParser.test.ts
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_number() {
        assert_eq!(number("$01a"), Ok(("a", 1)));
        assert_eq!(number("${0}"), Ok(("", 0)));
        assert_eq!(number("${00}a"), Ok(("a", 0)));
        assert!(number(" ${0}").is_err());
        assert!(number("${}").is_err());
        assert!(number("${-0}a").is_err());
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
        assert_eq!(
            placeholder("${1:true}").unwrap(),
            ("", Any::Placeholder(1, vec![Any::Text("true".into())]))
        );
    }

    #[test]
    fn can_variable() {
        assert_eq!(
            variable("${as:${wer:${2}}}").unwrap(),
            (
                "",
                Any::Variable(
                    "as",
                    V::Any(vec![Any::Variable("wer", V::Any(vec![Any::TabStop(2)]))])
                )
            )
        );
        assert_eq!(
            variable("${_3/.*/a/g}"),
            Ok((
                "",
                Any::Variable(
                    "_3",
                    V::Transform(Regex(".*"), vec![Format::Text("a")], Options("g"))
                )
            ))
        );
    }

    #[test]
    fn can_regex() {
        assert_eq!(regex("a"), Ok(("", Regex("a"))));
        assert_eq!(regex(r#"a\/"#), Ok(("", Regex(r#"a\/"#))));
        assert_eq!(regex(r#"a\\"#), Ok(("", Regex(r#"a\\"#))));
        assert_eq!(regex(r#"a/"#), Ok(("/", Regex("a"))));
        assert!(regex(r#"a\"#).is_err());
    }

    #[test]
    fn can_options() {
        assert_eq!(options("a"), Ok(("", Options("a"))));
        assert_eq!(options("a/"), Ok(("", Options("a/"))));
        assert_eq!(options("/"), Ok(("", Options("/"))));
        assert_eq!(options("g"), Ok(("", Options("g"))));
        assert_eq!(options("g}"), Ok(("}", Options("g"))));
    }

    #[test]
    fn can_format() {
        assert_eq!(formats("$2"), Ok(("", vec![Format::Matched(2)])));
        assert_eq!(formats("${3}"), Ok(("", vec![Format::Matched(3)])));
        assert_eq!(formats("${3:/upcase}"), Ok(("", vec![Format::Upcase(3)])));
        assert_eq!(
            formats("${3:+/upcase}"),
            Ok(("", vec![Format::If(3, "/upcase")]))
        );
        assert_eq!(
            formats("${3:?foo:bar}"),
            Ok(("", vec![Format::IfElse(3, "foo", "bar")]))
        );
        assert_eq!(formats("$"), Ok(("", vec![Format::Text("$")])));
        assert_eq!(formats(""), Ok(("", vec![])));
        assert_eq!(
            formats("${3}a/"),
            Ok(("/", vec![Format::Matched(3), Format::Text("a")]))
        );
    }

    #[test]
    fn can_text() {
        assert_eq!(text(r#"\$2"#), Ok(("", Any::Text(r#"$2"#.into()))));
        assert_eq!(text(""), Ok(("", Any::Text("".into()))));
        assert_eq!(text(" = {}\n\n"), Ok(("", Any::Text(" = {}\n\n".into()))));
        assert!(text(r#"$2"#).is_err());
    }

    #[test]
    fn can_parse() {
        assert_eq!(
            parse("if ${1:true} then\n\t$0\nend"),
            Some(Ast(vec![
                Any::Text("if ".into()),
                Any::Placeholder(1, vec![Any::Text("true".into())]),
                Any::Text(" then\n\t".into()),
                Any::TabStop(0),
                Any::Text("\nend".into())
            ]))
        );
        // XXX: Should be parsed as text without the need for escaping
        let a = parse("${1:className} = {}\n\n$1.${2:new} = function($3)\n\tlocal ${4:varName} = ${5:{}}\n\n\t${6: --code}\n\n\treturn $4\nend");
        let b = Some(Ast(vec![
            Any::Placeholder(1, vec![Any::Text("className".into())]),
            Any::Text(" = {}\n\n".into()),
            Any::TabStop(1),
            Any::Text(".".into()),
            Any::Placeholder(2, vec![Any::Text("new".into())]),
            Any::Text(" = function(".into()),
            Any::TabStop(3),
            Any::Text(")\n\tlocal ".into()),
            Any::Placeholder(4, vec![Any::Text("varName".into())]),
            Any::Text(" = ".into()),
            Any::Placeholder(5, vec![Any::Text("{".into())]),
            Any::Text("}\n\n\t".into()),
            Any::Placeholder(6, vec![Any::Text(" --code".into())]),
            Any::Text("\n\n\treturn ".into()),
            Any::TabStop(4),
            Any::Text("\nend".into()),
        ]));
        assert_eq!(a, b);
        dbg!(parse("if (${1:condition}) {\n\t${0}\n}"));
    }
}
