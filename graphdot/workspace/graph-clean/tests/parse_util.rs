//! Shared test-only ingest: a minimal parser for the exact DOT dialect the
//! tamarin web UI emits, reading a captured payload into the crate's [`Graph`]
//! model. Used by both the round-trip and allocator corpus tests. Not part of the
//! crate's public API — this is observation tooling, not the reimplementation.
#![allow(dead_code)]

use graph_clean::model::*;

/// Merge a plain-node legend that spans several physical lines into one logical
/// line, so every statement is a single element.
pub fn logical_lines(dot: &str) -> Vec<String> {
    let raw: Vec<&str> = dot.split('\n').collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        let line = raw[i];
        if line.contains("shape=\"plain\"") && !line.trim_end().ends_with("];") {
            let mut buf = String::from(line);
            i += 1;
            loop {
                buf.push('\n');
                buf.push_str(raw[i]);
                if raw[i].trim_end().ends_with("];") {
                    break;
                }
                i += 1;
            }
            out.push(buf);
            i += 1;
        } else {
            out.push(line.to_string());
            i += 1;
        }
    }
    out
}

pub fn parse(dot: &str) -> Graph {
    let lines = logical_lines(dot);
    let header = if lines.iter().any(|l| l == "packmode=\"cluster\";") {
        Header::Compact
    } else {
        Header::Simple
    };
    let mut idx = lines.iter().position(|l| l == "digraph \"G\" {").unwrap() + 1;
    let mut g = Graph::new(header);
    g.body = parse_block(&lines, &mut idx);
    g
}

fn parse_block(lines: &[String], idx: &mut usize) -> Vec<Stmt> {
    let mut out = Vec::new();
    while *idx < lines.len() {
        let line = lines[*idx].clone();
        let trimmed = line.trim_end();
        if trimmed == "}" {
            *idx += 1;
            return out;
        }
        if trimmed.is_empty() || is_block_attr(trimmed) {
            *idx += 1;
            continue;
        }
        if trimmed.starts_with("subgraph \"cluster_") {
            let label = trimmed
                .trim_start_matches("subgraph \"cluster_")
                .trim_end_matches("\" {")
                .to_string();
            *idx += 1;
            let color = capture_attr(lines, *idx, "color").unwrap();
            let body = parse_block(lines, idx);
            out.push(Stmt::Cluster(Cluster { label, color, body }));
            continue;
        }
        if trimmed == "{" {
            *idx += 1;
            let rank = capture_attr(lines, *idx, "rank").unwrap();
            let body = parse_block(lines, idx);
            out.push(Stmt::RankBlock(RankBlock { rank, body }));
            continue;
        }
        if line.contains(" -> ") {
            out.push(Stmt::Edge(parse_edge(&line)));
        } else {
            out.push(Stmt::Node(parse_node(&line)));
        }
        *idx += 1;
    }
    out
}

fn is_block_attr(l: &str) -> bool {
    if l.starts_with("node[") || l.starts_with("edge[") {
        return true;
    }
    if l.contains('[') {
        return false;
    }
    if let Some(eq) = l.find('=') {
        let key = &l[..eq];
        return key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) && l.ends_with(';');
    }
    false
}

fn capture_attr(lines: &[String], from: usize, key: &str) -> Option<String> {
    let needle = format!("{}=\"", key);
    for l in &lines[from..] {
        let t = l.trim();
        if t.starts_with(&needle) {
            let rest = &t[needle.len()..];
            return Some(rest.trim_end_matches("\";").trim_end_matches('"').to_string());
        }
        if t == "}" {
            break;
        }
    }
    None
}

fn parse_node(line: &str) -> Node {
    let l = line.trim_end();
    let br = l.find('[').unwrap();
    let id = l[..br].to_string();
    let attrs = &l[br + 1..l.rfind("];").unwrap()];
    if attrs.starts_with("shape=\"record\"") {
        let label_start = l.find(",label=\"").unwrap() + ",label=\"".len();
        let label_end = l.find("\",fillcolor=\"").unwrap();
        let label = &l[label_start..label_end];
        let fillcolor = extract(l, "fillcolor=\"");
        let fontcolor = extract(l, "fontcolor=\"");
        let role = extract(l, "role=\"");
        Node::record(
            id,
            Record { columns: parse_record_label(label), fillcolor, fontcolor, role: Role(role) },
        )
    } else if attrs.starts_with("shape=\"plain\"") {
        let start = l.find("label=<").unwrap() + "label=<".len();
        let end = l.rfind(">];").unwrap();
        Node::plain(id, &l[start..end])
    } else {
        let label_start = l.find("label=\"").unwrap() + "label=\"".len();
        let label_end = l.find("\",shape=\"").unwrap();
        let label = l[label_start..label_end].to_string();
        let shape = extract(l, "shape=\"");
        let color = if l.contains(",color=\"") { Some(extract(l, ",color=\"")) } else { None };
        if shape == "ellipse" {
            Node::ellipse(id, Ellipse { label, color })
        } else {
            Node::shaped(id, Shaped { label, shape, color })
        }
    }
}

fn extract(l: &str, key: &str) -> String {
    let s = l.find(key).unwrap() + key.len();
    let rest = &l[s..];
    let e = rest.find('"').unwrap();
    rest[..e].to_string()
}

fn parse_record_label(s: &str) -> Vec<Vec<Cell>> {
    let inner = &s[1..s.len() - 1];
    split_top(inner, '|')
        .into_iter()
        .map(|group| {
            let g = &group[1..group.len() - 1];
            split_top(g, '|').into_iter().map(|cell| parse_cell(&cell)).collect()
        })
        .collect()
}

fn parse_cell(cell: &str) -> Cell {
    let gt = find_unescaped(cell, '>').unwrap();
    let port = cell[1..gt].to_string();
    let text = cell[gt + 2..].to_string();
    Cell { port, text }
}

fn split_top(s: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut esc = false;
    for c in s.chars() {
        if esc {
            cur.push(c);
            esc = false;
            continue;
        }
        match c {
            '\\' => {
                cur.push(c);
                esc = true;
            }
            '{' => {
                depth += 1;
                cur.push(c);
            }
            '}' => {
                depth -= 1;
                cur.push(c);
            }
            _ if c == sep && depth == 0 => {
                parts.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    parts.push(cur);
    parts
}

fn find_unescaped(s: &str, target: char) -> Option<usize> {
    let mut esc = false;
    for (i, c) in s.char_indices() {
        if esc {
            esc = false;
            continue;
        }
        if c == '\\' {
            esc = true;
        } else if c == target {
            return Some(i);
        }
    }
    None
}

fn parse_edge(line: &str) -> Edge {
    let l = line.trim_end().trim_end_matches(';');
    let arrow = l.find(" -> ").unwrap();
    let src = parse_endpoint(&l[..arrow]);
    let rest = &l[arrow + 4..];
    let (dst_str, attrs) = match rest.find('[') {
        Some(b) => (&rest[..b], parse_attrs(&rest[b + 1..rest.rfind(']').unwrap()])),
        None => (rest, Vec::new()),
    };
    Edge { src, dst: parse_endpoint(dst_str), attrs }
}

fn parse_endpoint(s: &str) -> EndPoint {
    match s.find(':') {
        Some(c) => EndPoint::port(s[..c].to_string(), s[c + 1..].to_string()),
        None => EndPoint::node(s.to_string()),
    }
}

fn parse_attrs(s: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for part in s.split("\",") {
        let part = part.trim_end_matches('"');
        if let Some(eq) = part.find("=\"") {
            out.push((part[..eq].to_string(), part[eq + 2..].to_string()));
        }
    }
    out
}
