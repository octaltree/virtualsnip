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

//#[tokio::test]
// async fn can_calc() {
//    let req = Request {
//        highlight: Highlight {
//            base: "Comment".into()
//        },
//        sign: " ".into(),
//        start_line: 2,
//        cursor_line: 3,
//        lines: vec!["fn main(){".into(), "    if a == b {".into()],
//        snippets: vec![vec![
//            Node::Text(NodeText {
//                value: "if ".into()
//            }),
//            Node::Placeholder(NodePlaceholder {
//                children: vec![Node::Text(NodeText {
//                    value: "condition".into()
//                })]
//            }),
//            Node::Text(NodeText {
//                value: " {\n    ".into()
//            }),
//            Node::Placeholder(NodePlaceholder {
//                children: vec![Node::Text(NodeText {
//                    value: "unimplemented!();".into()
//                })]
//            }),
//            Node::Text(NodeText {
//                value: "\n}".into()
//            }),
//        ]]
//    };
//    let y = calc(&req).await;
//    assert_eq!(
//        y,
//        Response {
//            texts: vec![Text {
//                line: 3,
//                chunks: vec![(
//                    Cow::Borrowed(" unimplemented!();\n}"),
//                    Cow::Borrowed("Comment")
//                )]
//            }]
//        }
//    );
//}

// TODO: escaped "\n"
//#[tokio::test]
// async fn main() {
//    let req = {
//        use Node::{Placeholder, Text};
//        Request {
//            highlight: Highlight {
//                base: "Comment".into()
//            },
// sign: " ".into(),
//            lines: vec!["fn main() {".into(), "    if true { aaaasd".into()],
//            start_line: 0,
//            cursor_line: 1,
//            snippets: vec![
//                vec![
//                    Text(NodeText {
//                        value: "debug_assert_eq!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ", ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ");".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "concat_idents!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "include_bytes!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\")".into()
//                    }),
//                ],
//                vec![Text(NodeText {
//                    value: "unimplemented!()".into()
//                })],
//                vec![
//                    Text(NodeText {
//                        value: "extern crate ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ";".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "debug_assert!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ");".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "thread_local!(static ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "STATIC".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ": ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " = ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "init".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ");".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "struct ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Name".into()
//                        })]
//                    }),
//                    Text(NodeText { value: "(".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ");".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "macro_rules! ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    (".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: ") => (".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: ")\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "struct ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Name".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ";".into() }),
//                ],
//                vec![Text(NodeText {
//                    value: "module_path!()".into()
//                })],
//                vec![
//                    Text(NodeText {
//                        value: "format_args!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\")".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "unreachable!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "include_str!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\")".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "option_env!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\")".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "extern \"C\" {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "// add code here".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "impl ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Trait".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " for ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "// add code here".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![inline]\\npub fn ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "() {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "stringify!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "assert_eq!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ", ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ");".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![macro_use(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "while let ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Some(pat)".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " = ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "expr".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "extern \"C\" fn ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText { value: "(".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "arg".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ": ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: ") -> ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "RetType".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "// add code here".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "mod ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "// add code here".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![cfg_attr(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ", ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![Text(NodeText {
//                    value: "#!vec![no_core]".into()
//                })],
//                vec![
//                    Text(NodeText {
//                        value: "#!vec![feature(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "include!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\");".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "writeln!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: ", \"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\")".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "println!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\");".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![derive(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![derive(Debug)]\\nstruct ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Name".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "field".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ": ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "static ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "STATIC".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ": ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " = ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "init".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ";".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "if let ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Some(pat)".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " = ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "expr".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![Text(NodeText {
//                    value: "#!vec![no_std]".into()
//                })],
//                vec![Text(NodeText {
//                    value: "column!()".into()
//                })],
//                vec![
//                    Text(NodeText {
//                        value: "concat!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "assert!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ");".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "format!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\")".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "write!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: ", \"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\")".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "print!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\");".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "trait ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Name".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "// add code here".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}\\n".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "const ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "CONST".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ": ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " = ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "init".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ";".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#!vec![allow(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "match ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "expr".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Some(expr)".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " => ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "expr".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: ",\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "None".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " => ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "expr".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: ",\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "while ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "condition".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![bench]\\nfn ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "(b: &mut test::Bencher) {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![
//                            Text(NodeText {
//                                value: "b.iter(|| ".into()
//                            }),
//                            Placeholder(NodePlaceholder {
//                                children: vec![Text(NodeText {
//                                    value: "/* benchmark code */".into()
//                                })]
//                            }),
//                            Text(NodeText { value: ")".into() }),
//                        ]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "panic!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\");".into()
//                    }),
//                ],
//                vec![Text(NodeText {
//                    value: "line!()".into()
//                })],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![repr(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "else {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "type ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Alias".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " = ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ";".into() }),
//                ],
//                vec![Text(NodeText {
//                    value: "file!()".into()
//                })],
//                vec![
//                    Text(NodeText {
//                        value: "Some(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "impl ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "// add code here".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "loop {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "fn main() {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#!vec![warn(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![derive(Debug)]\\nenum ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Name".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Variant1".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: ",\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Variant2".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: ",\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#!vec![deny(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![test]\\nfn ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "() {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "cfg!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "for ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "pat".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " in ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "expr".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "vec!vec![".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: "]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "env!(\"".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText {
//                        value: "\")".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "#vec![cfg(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")]".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "mod ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ";".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "try!(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "let ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "pat".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " = ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "expr".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ";".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "Err(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText { value: "".into() })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "Ok(".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "result".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ")".into() }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "fn ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "name".into()
//                        })]
//                    }),
//                    Text(NodeText { value: "(".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "arg".into()
//                        })]
//                    }),
//                    Text(NodeText { value: ": ".into() }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "Type".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: ")-> ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "RetType".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//                vec![
//                    Text(NodeText {
//                        value: "if ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "condition".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: " {\\n    ".into()
//                    }),
//                    Placeholder(NodePlaceholder {
//                        children: vec![Text(NodeText {
//                            value: "unimplemented!();".into()
//                        })]
//                    }),
//                    Text(NodeText {
//                        value: "\\n}".into()
//                    }),
//                ],
//            ]
//        }
//    };
//    let y = calc(&req).await;
//    assert_eq!(
//        y,
//        Response {
//            texts: vec![
//                Text {
//                    line: 0,
//                    chunks: vec![(
//                        Cow::Borrowed("unimplemented!();\n}"),
//                        Cow::Borrowed("Comment")
//                    )]
//                },
//                Text {
//                    line: 1,
//                    chunks: vec![(
//                        Cow::Borrowed("unimplemented!();\n}"),
//                        Cow::Borrowed("Comment")
//                    )]
//                }
//            ]
//        }
//    );
//}
