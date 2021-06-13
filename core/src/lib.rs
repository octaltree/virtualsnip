pub mod vs_snippet;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, io::Write, ops::Deref};

#[derive(Debug, Deserialize)]
pub struct Request {
    highlight: Highlight,
    sign: String,
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

// TODO: async
pub async fn calc(req: &Request) -> Response<'_> {
    if req.snippets.is_empty() {
        return Response::default();
    }
    let num = req.cursor_line - req.start_line + 1;
    let before_cursor_inclusive = &req.lines[..num];
    let matched = r#match(before_cursor_inclusive, &req.snippets);
    let mut texts = Vec::new();
    for l in req.start_line..=req.cursor_line {
        let i = l - req.start_line;
        let nodes = matched[i];
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
        let text = Text { line: l, chunks };
        texts.push(text);
    }
    Response { texts }
}

fn r#match<'a>(buf: &[String], snippets: &'a [Vec<Node>]) -> Vec<&'a [Node]> {
    let snips: Vec<&[Node]> = snippets.iter().map(Deref::deref).map(first_text).collect();
    let mut nodes_for_lines = Vec::with_capacity(buf.len());
    for l in buf {
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
                    let tail = &s[j..];
                    if tail.iter().all(|n| !n.is_text()) {
                        &[]
                    } else {
                        tail
                    }
                }
            } else {
                &[]
            }
        };
        nodes_for_lines.push(nodes_for_this_line);
    }
    nodes_for_lines
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
