use serde::{Deserialize, Serialize};
use std::{borrow::Cow, io::Write, ops::Deref};

#[derive(Debug, Deserialize)]
pub struct Request {
    highlight: Highlight,
    lines: Vec<String>,
    start_line: usize,
    cursor_line: usize,
    snippets: Vec<Vec<Node>>
}

#[derive(Debug, Deserialize)]
struct Highlight {
    base: String
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Node {
    Variable(NodeVariable),
    Placeholder(NodePlaceholder),
    Text(NodeText)
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NodeText {
    value: String
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NodePlaceholder {
    children: Vec<Node>
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct NodeVariable {
    children: Vec<Node>
}

#[derive(Debug, Serialize, Default, PartialEq, Eq)]
pub struct Response<'a> {
    texts: Vec<Text<'a>>
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Text<'a> {
    line: usize,
    chunks: Vec<(Cow<'a, str>, Cow<'a, str>)>
}

pub fn deserialize_request(s: &str) -> Request { serde_json::from_str(s).unwrap() }

pub fn write_response<W: Write>(w: W, resp: &Response<'_>) {
    serde_json::to_writer(w, resp).unwrap()
}

pub async fn calc(req: &Request) -> Response<'_> {
    if req.snippets.is_empty() {
        return Response::default();
    }
    let num = req.cursor_line - req.start_line + 1;
    let before_cursor_inclusive = &req.lines[..num];
    let matched = r#match(before_cursor_inclusive, &req.snippets);
    let texts: Vec<_> = (req.start_line..=req.cursor_line)
        .map(|l| {
            let i = l - req.start_line;
            let nodes = matched[i];
            let chunks = vec![(
                Cow::Owned(
                    nodes
                        .iter()
                        .map(|n| text(n).to_string())
                        .collect::<Vec<_>>()
                        .join("")
                ),
                Cow::Borrowed(&req.highlight.base as &str)
            )];
            Text { line: l, chunks }
        })
        .collect();
    Response { texts }
}

fn r#match<'a>(buf: &[String], snippets: &'a [Vec<Node>]) -> Vec<&'a [Node]> {
    let snips: Vec<&[Node]> = snippets.iter().map(Deref::deref).map(first_text).collect();
    buf.iter()
        .map(|l| {
            let founds: Vec<_> = snips.iter().map(|nodes| find(l, nodes)).collect();
            let max: Option<(_, _)> = {
                let mut max = 0.;
                let mut v = None;
                for (n, f) in snips.iter().zip(founds.iter()) {
                    if f.num == 0 || f.hit == 0 || f.hit == f.num {
                        continue;
                    }
                    let score = f.hit as f64 / f.num as f64;
                    if max < score {
                        max = score;
                        v = Some((n, f));
                    }
                }
                v
            };
            #[allow(clippy::let_and_return)]
            let nodes_for_this_line = {
                if let Some((s, f)) = max {
                    if f.hit < f.num_first {
                        &s
                    } else {
                        let mut j = s.len();
                        let mut k = 0;
                        for i in 1..s.len() {
                            j = i;
                            if k >= f.hit - f.num_first {
                                break;
                            }
                            if s[i].is_text() {
                                k += 1
                            }
                        }
                        &s[j..]
                    }
                } else {
                    &[]
                }
            };
            nodes_for_this_line
        })
        .collect()
}

fn first_text(nodes: &[Node]) -> &[Node] {
    if let Some((i, _)) = nodes.iter().enumerate().find(|(_, n)| n.is_text()) {
        &nodes[i..]
    } else {
        &[]
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Found {
    num: usize,
    hit: usize,
    num_first: usize
}

fn find(line: &str, nodes: &[Node]) -> Found {
    if nodes.is_empty() || line.is_empty() {
        return Found::default();
    }
    // the first node is text type
    let first = match &nodes[0] {
        Node::Text(n) => n,
        _ => unreachable!()
    };
    let fs = first.value.trim().split(char::is_whitespace);
    let num_first = fs.clone().count();
    let rest: Vec<_> = nodes[1..]
        .iter()
        .filter_map(|n| match n {
            Node::Text(n) => Some(n),
            _ => None
        })
        .collect();
    let num = fs.clone().count() + rest.len();
    let chars: Vec<char> = line.chars().collect();
    let mut hit = 0;
    let mut cur = 0;
    for word in fs {
        if let Some(i) = contains(&chars[cur..], word) {
            hit += 1;
            cur = i + word.chars().count();
        } else {
            return Found {
                hit,
                num,
                num_first
            };
        }
    }
    for n in rest {
        let word = n.value.trim();
        if let Some(i) = contains(&chars[cur..], word) {
            hit += 1;
            cur = i + word.chars().count();
        } else {
            return Found {
                hit,
                num,
                num_first
            };
        }
    }
    Found {
        hit,
        num,
        num_first
    }
}

fn contains(sentence: &[char], words: &str) -> Option<usize> {
    let wc: Vec<char> = words.chars().collect();
    if sentence.len() < wc.len() {
        return None;
    }
    for i in 0..(sentence.len() - wc.len() + 1) {
        if sentence[i..i + wc.len()] == wc {
            return Some(i);
        }
    }
    None
}

impl Node {
    fn is_text(&self) -> bool { matches!(self, Node::Text(_)) }
}

fn text(node: &Node) -> Cow<'_, str> {
    let children = match node {
        Node::Text(t) => return Cow::Borrowed(&t.value),
        Node::Variable(n) => &n.children,
        Node::Placeholder(n) => &n.children
    };
    Cow::Owned(children.iter().map(text).collect::<Vec<_>>().join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let resp = Response {
            texts: vec![Text {
                line: 3,
                chunks: vec![(Cow::Borrowed("foo"), Cow::Borrowed("Comment"))]
            }]
        };
        let s = serde_json::to_string(&resp).unwrap();
        assert_eq!(s, r#"{"texts":[{"line":3,"chunks":[["foo","Comment"]]}]}"#)
    }

    #[test]
    fn deserialize() {
        let s = r#"{"type":"text","value":"a"}"#;
        let n: Node = serde_json::from_str(s).unwrap();
        assert_eq!(n, Node::Text(NodeText { value: "a".into() }));
    }

    #[test]
    fn can_find() {
        let snippet = &[
            Node::Text(NodeText {
                value: "if ".into()
            }),
            Node::Placeholder(NodePlaceholder {
                children: vec![Node::Text(NodeText {
                    value: "condition".into()
                })]
            }),
            Node::Text(NodeText {
                value: " {\n    ".into()
            }),
            Node::Placeholder(NodePlaceholder {
                children: vec![Node::Text(NodeText {
                    value: "unimplemented!();".into()
                })]
            }),
            Node::Text(NodeText {
                value: "\n}".into()
            })
        ];
        let found = find("    if true {", snippet);
        assert_eq!(
            found,
            Found {
                num: 3,
                hit: 2,
                num_first: 1
            }
        );
    }
    #[test]
    fn can_contains() {
        assert_eq!(
            contains(&"    if ".chars().collect::<Vec<_>>(), "if"),
            Some(4)
        );
        assert_eq!(
            contains(&"    if true {\n".chars().collect::<Vec<_>>(), "{\n"),
            Some(12)
        );
    }

    #[test]
    fn can_match() {
        let snippets = &[vec![
            Node::Text(NodeText {
                value: "if ".into()
            }),
            Node::Placeholder(NodePlaceholder {
                children: vec![Node::Text(NodeText {
                    value: "condition".into()
                })]
            }),
            Node::Text(NodeText {
                value: " {\n    ".into()
            }),
            Node::Placeholder(NodePlaceholder {
                children: vec![Node::Text(NodeText {
                    value: "unimplemented!();".into()
                })]
            }),
            Node::Text(NodeText {
                value: "\n}".into()
            }),
        ]];
        let before_cursor_inclusive = &["fn main(){".into(), "    if a == b {".into()];
        assert_eq!(
            r#match(before_cursor_inclusive, snippets),
            vec![
                vec![],
                vec![
                    Node::Placeholder(NodePlaceholder {
                        children: vec![Node::Text(NodeText {
                            value: "unimplemented!();".into()
                        })]
                    }),
                    Node::Text(NodeText {
                        value: "\n}".into()
                    }),
                ],
            ]
        );
    }

    #[tokio::test]
    async fn can_calc() {
        let req = Request {
            highlight: Highlight {
                base: "Comment".into()
            },
            start_line: 2,
            cursor_line: 3,
            lines: vec!["fn main(){".into(), "    if a == b {".into()],
            snippets: vec![vec![
                Node::Text(NodeText {
                    value: "if ".into()
                }),
                Node::Placeholder(NodePlaceholder {
                    children: vec![Node::Text(NodeText {
                        value: "condition".into()
                    })]
                }),
                Node::Text(NodeText {
                    value: " {\n    ".into()
                }),
                Node::Placeholder(NodePlaceholder {
                    children: vec![Node::Text(NodeText {
                        value: "unimplemented!();".into()
                    })]
                }),
                Node::Text(NodeText {
                    value: "\n}".into()
                }),
            ]]
        };
        let y = calc(&req).await;
        assert_eq!(
            y,
            Response {
                texts: vec![
                    Text {
                        line: 2,
                        chunks: vec![(Cow::Borrowed(""), Cow::Borrowed("Comment"))]
                    },
                    Text {
                        line: 3,
                        chunks: vec![(
                            Cow::Borrowed("unimplemented!();\n}"),
                            Cow::Borrowed("Comment")
                        )]
                    }
                ]
            }
        );
    }

    #[tokio::test]
    async fn main() {
        let req = {
            use Node::{Placeholder, Text};
            Request {
                highlight: Highlight {
                    base: "Comment".into()
                },
                lines: vec!["fn main() {".into(), "    if ".into()],
                start_line: 0,
                cursor_line: 1,
                snippets: vec![
                    vec![
                        Text(NodeText {
                            value: "debug_assert_eq!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ", ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ");".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "concat_idents!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "include_bytes!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\")".into()
                        }),
                    ],
                    vec![Text(NodeText {
                        value: "unimplemented!()".into()
                    })],
                    vec![
                        Text(NodeText {
                            value: "extern crate ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ";".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "debug_assert!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ");".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "thread_local!(static ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ": ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " = ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ");".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "struct ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: "(".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ");".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "macro_rules! ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    (".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ") => (".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ")\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "struct ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ";".into() }),
                    ],
                    vec![Text(NodeText {
                        value: "module_path!()".into()
                    })],
                    vec![
                        Text(NodeText {
                            value: "format_args!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\")".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "unreachable!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "include_str!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\")".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "option_env!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\")".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "extern \"C\" {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "impl ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " for ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![inline]\\npub fn ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "() {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "stringify!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "assert_eq! (".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ", ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ");".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![macro_use(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "while let ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " = ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "extern \"C\" fn ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: "(".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ": ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ") -> ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "mod ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![cfg_attr(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ", ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![Text(NodeText {
                        value: "#!vec![no_core]".into()
                    })],
                    vec![
                        Text(NodeText {
                            value: "#!vec![feature(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "include!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\");".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "writeln!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ", \"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: "\"".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "println!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\");".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![derive(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![derive(Debug)]\\nstruct ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ": ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "static ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ": ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " = ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ";".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "if let ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " = ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![Text(NodeText {
                        value: "#!vec![no_std]".into()
                    })],
                    vec![Text(NodeText {
                        value: "column!()".into()
                    })],
                    vec![
                        Text(NodeText {
                            value: "concat!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "assert!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ");".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "format!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\")".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "write!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ", \"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\")".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "print!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\");".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "trait ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}\\n".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "const ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ": ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " = ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ";".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#!vec![allow(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "match ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " => ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ",\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " => ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ",\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "while ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![bench]\\nfn ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "(b: &mut test::Bencher) {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "panic!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\");".into()
                        }),
                    ],
                    vec![Text(NodeText {
                        value: "line!()".into()
                    })],
                    vec![
                        Text(NodeText {
                            value: "#vec![repr(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "else {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "type ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " = ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ";".into() }),
                    ],
                    vec![Text(NodeText {
                        value: "file!()".into()
                    })],
                    vec![
                        Text(NodeText {
                            value: "Some(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "impl ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "loop {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "fn main() {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#!vec![warn(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![derive(Debug)]\\nenum ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ",\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ",\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#!vec![deny(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![test]\\nfn ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "() {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "cfg!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "for ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " in".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "vec!vec![".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: "]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "env!(\"".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\")".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "#vec![cfg(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")]".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "mod ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ";".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "try!(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "let ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " = ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ";".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "Err(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "Ok(".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ")".into() }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "fn ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: "(".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText { value: ": ".into() }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: ") -> ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                    vec![
                        Text(NodeText {
                            value: "if ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: " {\\n    ".into()
                        }),
                        Placeholder(NodePlaceholder { children: vec![] }),
                        Text(NodeText {
                            value: "\\n}".into()
                        }),
                    ],
                ]
            }
        };
        let y = calc(&req).await;
        assert_eq!(
            y,
            Response {
                texts: vec![
                    Text {
                        line: 0,
                        chunks: vec![(Cow::Borrowed(""), Cow::Borrowed("Comment"))]
                    },
                    Text {
                        line: 1,
                        chunks: vec![(
                            Cow::Borrowed("unimplemented!();\n}"),
                            Cow::Borrowed("Comment")
                        )]
                    }
                ]
            }
        );
    }
}
