pub mod vs_snippet;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::{Read, Write},
    ops::Deref
};

#[derive(Debug, Deserialize)]
pub struct Request {
    highlight: Highlight,
    sign: String,
    lines: Vec<String>,
    start_line: usize,
    cursor_line: usize,
    // snippets: Vec<Vec<Node>>
    sources: Vec<Vec<Snippet>>
}

#[derive(Debug, Deserialize)]
pub struct Snippet {
    body: Vec<String>
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

pub fn read_request<R: Read>(r: R) -> Request { serde_json::from_reader(r).unwrap() }

pub fn write_response<W: Write>(w: W, resp: &Response<'_>) {
    serde_json::to_writer(w, resp).unwrap()
}

pub fn calc(req: &Request) -> Response<'_> {
    let snippets: Vec<_> = req
        .sources
        .iter()
        .flat_map(|snippets| snippets.iter())
        .par_bridge()
        .filter_map(nodes)
        .collect();
    if snippets.is_empty() {
        return Response::default();
    }
    let num = req.cursor_line - req.start_line + 1;
    let before_cursor_inclusive = &req.lines[..num];
    let matched = r#match(req.start_line, before_cursor_inclusive, &snippets);
    let mut texts = Vec::new();
    for (l, i) in matched {
        let nodes = l;
        if nodes.is_empty() {
            continue;
        }
        let chunks = vec![(
            Cow::Owned(format!(
                "{}{}",
                req.sign,
                nodes
                    .iter()
                    .map(|n| text(n).to_string())
                    .collect::<Vec<_>>()
                    .join("")
            )),
            Cow::Borrowed(&req.highlight.base as &str)
        )];
        let text = Text { line: i, chunks };
        texts.push(text);
    }
    Response { texts }
}

fn nodes(snip: &Snippet) -> Option<Vec<Node>> {
    let b = snip.body.join("\n");
    let ast = vs_snippet::parse(&b)?;
    Some(ast.0.into_iter().map(node_from_ast).collect())
}

fn node_from_ast(any: vs_snippet::Any<'_>) -> Node {
    match any {
        vs_snippet::Any::TabStop(_) => Node::Placeholder(NodePlaceholder { children: vec![] }),
        vs_snippet::Any::Placeholder(_, cs) => Node::Placeholder(NodePlaceholder {
            children: cs.into_iter().map(node_from_ast).collect()
        }),
        vs_snippet::Any::Choice(_, _) => Node::Placeholder(NodePlaceholder { children: vec![] }),
        vs_snippet::Any::Variable(_, vs_snippet::V::Any(cs)) => Node::Variable(NodeVariable {
            children: cs.into_iter().map(node_from_ast).collect()
        }),
        vs_snippet::Any::Variable(_, _) => Node::Variable(NodeVariable { children: vec![] }),
        vs_snippet::Any::Text(s) => Node::Text(NodeText { value: s })
    }
}

fn r#match<'a>(
    start_line: usize,
    buf: &[String],
    snippets: &'a [Vec<Node>]
) -> Vec<(&'a [Node], usize)> {
    let snips: Vec<&[Node]> = snippets.iter().map(Deref::deref).map(first_text).collect();
    buf.iter()
        .zip(start_line..)
        .par_bridge()
        .map(|(l, i)| {
            let founds: Vec<_> = snips.iter().map(|nodes| find(l, nodes)).collect();
            let max: Option<(_, _)> = {
                let mut max = 0.;
                let mut v = None;
                for (n, f) in snips.iter().zip(founds.iter()) {
                    if f.num == 0 || f.hit == 0 {
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
            let nodes_for_this_line = max.map(tail_excluding_matches).unwrap_or_default();
            (nodes_for_this_line, i)
        })
        .collect()
}

fn tail_excluding_matches<'a, 'b>(found: (&'a &'b [Node], &'a Found)) -> &'b [Node] {
    let (s, f) = found;
    if f.hit < f.num_first {
        return s;
    }
    let ns = match s.len() {
        0 | 1 => return &[],
        _ => &s[1..]
    };
    let matches = f.hit - f.num_first;
    let idx = {
        let mut k = 0;
        let mut m = 0;
        for (i, n) in ns.iter().enumerate() {
            if m >= matches {
                break;
            }
            k = i + 1;
            if n.is_text() {
                m += 1;
            }
        }
        k
    };
    let tail = &ns[idx..];
    if tail.iter().all(|n| !n.is_text()) {
        &[]
    } else {
        tail
    }
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
            // Even if one letter exists, it's unlikely.
            if word.len() > 1 {
                hit += 1;
            }
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
mod tests;
