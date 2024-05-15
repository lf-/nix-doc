// SPDX-FileCopyrightText: 2024 Jade Lovelace
//
// SPDX-License-Identifier: BSD-2-Clause OR MIT

//! library components of nix-doc
pub mod pprint;
pub mod tags;
pub mod threadpool;

use crate::pprint::pprint_args;
use crate::threadpool::ThreadPool;

use colorful::{Color, Colorful};
use regex::Regex;
use rnix::types::{AttrSet, EntryHolder, Ident, Lambda, TokenWrapper, TypedNode};
use rnix::SyntaxKind::*;
use rnix::{NodeOrToken, SyntaxNode, TextUnit, WalkEvent, AST};
use walkdir::{DirEntry, WalkDir};

use std::fs;
use std::iter;
use std::path::Path;
use std::sync::mpsc::channel;
use std::{fmt::Display, str};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DOC_INDENT: usize = 3;

/// Max size of files we will consider searching. It takes a long time to parse 300k lines of nix
/// in hackage-packages.nix and no files this big will have search results in them as they
/// categorically do not contain functions. 200k bytes is ~7.5k lines
const MAX_FILE_SIZE: u64 = 200_000;

struct SearchResult {
    /// Name of the function
    identifier: String,

    /// Dedented documentation comments
    doc: String,

    /// Parameter block for the function
    param_block: String,

    /// Start of the definition of the function
    defined_at_start: usize,
}

fn find_line(file: &str, pos: usize) -> usize {
    file[..pos].lines().count()
}

fn find_pos(file: &str, line: usize, col: usize) -> usize {
    let mut lines = 1;
    let mut line_start = 0;
    for (count, ch) in file.chars().enumerate() {
        if ch == '\n' {
            lines += 1;
            line_start = count;
        }
        if lines == line && count - line_start == col {
            return count;
        }
    }
    unreachable!();
}

impl SearchResult {
    fn format<P: Display>(&self, filename: P, line: usize) -> String {
        format!(
            "{}\n{} = {}\n# {}",
            indented(&self.doc, DOC_INDENT),
            self.identifier.as_str().white().bold(),
            self.param_block,
            format!("{}:{}", filename, line).as_str(),
        )
    }
}

/// Should the given path be searched?
/// TODO: support globbing for files e.g. with lib in their name to improve perf significantly
///       or avoid looking in absurdly large files like hackage.nix
pub fn is_searchable(fname: &Path) -> bool {
    fname.to_str().map(|s| s.ends_with(".nix")).unwrap_or(false)
}

/// Runs a search for files matching the regex `matching`. Returns a list of such results with the
/// associated file contents
fn search_file(file: &Path, matching: &Regex) -> Result<Vec<(SearchResult, usize)>> {
    // don't bother searching files that are so large they must be generated
    let length = fs::metadata(file)?.len();
    if length > MAX_FILE_SIZE {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(file)?;
    let ast = rnix::parse(&content).as_result()?;
    let results = search_ast(&matching, &ast);

    Ok(results
        .into_iter()
        .map(|res| {
            let line = find_line(&content, res.defined_at_start);
            (res, line)
        })
        .collect::<Vec<_>>())
}

/// Is a file hidden or a unicode decode error?
/// Let's not consider it.
pub fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s != "." && s.starts_with('.') || s == "target")
        .unwrap_or(true)
}

/// Search the `dir` for files with function definitions matching `matching`
pub fn search<F>(dir: &Path, matching: Regex, should_search: F)
where
    F: Fn(&Path) -> bool,
{
    let pool = ThreadPool::default();
    let (tx, rx) = channel();

    //println!("searching {}", dir.display());
    for direntry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
        .filter_map(|e| e.ok())
        .filter(|e| should_search(e.path()) && e.path().is_file())
    {
        let my_tx = tx.clone();
        let matching = matching.clone();
        pool.push(move || {
            //println!("{}", direntry.path().display());
            let results = search_file(direntry.path(), &matching);
            if let Err(err) = results {
                eprintln!("Failure handling {}: {}", direntry.path().display(), err);
                return;
            }
            let results = results.unwrap();

            let formatted = results
                .iter()
                .map(|(result, line)| result.format(direntry.path().display(), *line))
                .collect::<Vec<_>>();
            if !formatted.is_empty() {
                my_tx
                    .send(formatted)
                    .expect("failed to send messages to display");
            }
        });
    }

    drop(tx);
    pool.done();

    let line = iter::repeat("─")
        .take(45)
        .collect::<String>()
        .color(Color::Grey27);
    let mut is_first = true;

    while let Ok(results) = rx.recv() {
        for result in results {
            if !is_first {
                println!("{}", &line);
            } else {
                is_first = false;
            }
            println!("{}", result);
        }
    }
}

/// Searches the given AST for functions called `identifier`
fn search_ast(identifier: &Regex, ast: &AST) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for ev in ast.node().preorder_with_tokens() {
        match ev {
            WalkEvent::Enter(enter) => {
                //println!("enter {:?}", &enter);
                if let Some(set) = enter.into_node().and_then(AttrSet::cast) {
                    results.extend(visit_attrset(identifier, &set));
                }
            }
            WalkEvent::Leave(_leave) => {
                //println!("leave {:?}", &leave);
            }
        }
    }
    results
}

/// Emits a string `s` indented by `indent` spaces
fn indented(s: &str, indent: usize) -> String {
    let indent_s = iter::repeat(' ').take(indent).collect::<String>();
    s.split('\n')
        .map(|line| indent_s.clone() + line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Cleans up a single line, erasing prefix single line comments but preserving indentation
fn cleanup_single_line<'a>(s: &'a str) -> &'a str {
    let mut cmt_new_start = 0;
    let mut iter = s.char_indices().peekable();
    while let Some((idx, ch)) = iter.next() {
        // peek at the next character, with an explicit '\n' as "next character" at end of line
        let (_, next_ch) = iter.peek().unwrap_or(&(0, '\n'));

        // if we find a character, save the byte position after it as our new string start
        if ch == '#' || ch == '*' && next_ch.is_whitespace() {
            cmt_new_start = idx + 1;
            break;
        }
        // if, instead, we are on a line with no starting comment characters, leave it alone as it
        // will be handled by dedent later
        if !ch.is_whitespace() {
            break;
        }
    }
    &s[cmt_new_start..]
}

/// Erases indents in comments. This is *almost* a normal dedent function, but it starts by looking
/// at the second line if it can.
fn dedent_comment(s: &str) -> String {
    let mut whitespaces = 0;
    let mut lines = s.lines();
    let first = lines.next();

    // scan for whitespace
    for line in lines.chain(first) {
        let line_whitespace = line.chars().take_while(|ch| ch.is_whitespace()).count();

        if line_whitespace != line.len() {
            // a non-whitespace line, perfect for taking whitespace off of
            whitespaces = line_whitespace;
            break;
        }
    }

    // maybe the first considered line we found was indented further, so let's look for more lines
    // that might have a shorter indent. In the case of one line, do nothing.
    for line in s.lines().skip(1) {
        let line_whitespace = line.chars().take_while(|ch| ch.is_whitespace()).count();

        if line_whitespace != line.len() {
            whitespaces = line_whitespace.min(whitespaces);
        }
    }

    // delete up to `whitespaces` whitespace characters from each line and reconstitute the string
    let mut out = String::new();
    for line in s.lines() {
        let content_begin = line.find(|ch: char| !ch.is_whitespace()).unwrap_or(0);
        out.push_str(&line[content_begin.min(whitespaces)..]);
        out.push('\n');
    }

    out.truncate(out.trim_end_matches('\n').len());
    out
}

/// Deletes whitespace and leading comment characters
///
/// Oversight we are choosing to ignore: if you put # characters at the beginning of lines in a
/// multiline comment, they will be deleted.
fn cleanup_comments<S: AsRef<str>, I: DoubleEndedIterator<Item = S>>(comment: &mut I) -> String {
    dedent_comment(
        &comment
            .rev()
            .map(|small_comment| {
                small_comment
                    .as_ref()
                    // space before multiline start
                    .trim_start()
                    // multiline starts
                    .trim_start_matches("/*")
                    // trailing so we can grab multiline end
                    .trim_end()
                    // multiline ends
                    .trim_end_matches("*/")
                    // extra space that was in the multiline
                    .trim()
                    .split('\n')
                    // erase single line comments and such
                    .map(cleanup_single_line)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// Get the docs for a specific function
pub fn get_function_docs(filename: &str, line: usize, col: usize) -> Option<String> {
    let content = fs::read(filename).ok()?;
    let decoded = str::from_utf8(&content).ok()?;
    let pos = find_pos(&decoded, line, col);
    let rowan_pos = TextUnit::from_usize(pos);
    let tree = rnix::parse(decoded);

    let mut lambda = None;
    for node in tree.node().preorder() {
        match node {
            WalkEvent::Enter(n) => {
                if n.text_range().start() >= rowan_pos && n.kind() == NODE_LAMBDA {
                    lambda = Lambda::cast(n);
                    break;
                }
            }
            WalkEvent::Leave(_) => (),
        }
    }
    let lambda = lambda?;
    let res = visit_lambda("func".to_string(), pos, &lambda);
    Some(res.format(filename, line))
}

fn visit_lambda(name: String, defined_at_start: usize, lambda: &Lambda) -> SearchResult {
    // grab the arguments
    let param_block = pprint_args(&lambda);

    // find the doc comment
    let comment = find_comment(lambda.node().clone()).unwrap_or_else(|| "".to_string());

    SearchResult {
        identifier: name,
        doc: comment,
        param_block,
        defined_at_start,
    }
}

fn visit_attrset(id_needle: &Regex, set: &AttrSet) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for entry in set.entries() {
        if let Some(lambda) = entry.value().and_then(Lambda::cast) {
            if let Some(attr) = entry.key() {
                let ident = attr.path().last().and_then(Ident::cast);
                let defined_at_start = ident
                    .as_ref()
                    .map(|i| i.node().text_range().start().to_usize());

                let ident_name = ident.as_ref().map(|id| id.as_str());

                if ident_name.map(|id| id_needle.is_match(id)) != Some(true) {
                    // rejected, not matching our pattern
                    continue;
                }
                let ident_name = ident_name.unwrap();

                let res = visit_lambda(ident_name.to_string(), defined_at_start.unwrap(), &lambda);
                if !res.doc.is_empty() {
                    results.push(res);
                }
            }
        }
    }
    results
}

fn find_comment(node: SyntaxNode) -> Option<String> {
    let mut node = NodeOrToken::Node(node);
    let mut comments = Vec::new();
    loop {
        loop {
            if let Some(new) = node.prev_sibling_or_token() {
                node = new;
                break;
            } else {
                node = NodeOrToken::Node(node.parent()?);
            }
        }

        match node.kind() {
            TOKEN_COMMENT => match &node {
                NodeOrToken::Token(token) => comments.push(token.text().clone()),
                NodeOrToken::Node(_) => unreachable!(),
            },
            // This stuff is found as part of `the-fn = f: ...`
            // here:                           ^^^^^^^^
            NODE_KEY | TOKEN_ASSIGN => (),
            t if t.is_trivia() => (),
            _ => break,
        }
    }
    let doc = cleanup_comments(&mut comments.iter().map(|c| c.as_str()));
    Some(doc).filter(|it| !it.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytepos() {
        let fakefile = "abc\ndef\nghi";
        assert_eq!(find_pos(fakefile, 2, 2), 5);
    }

    #[test]
    fn test_comment_stripping() {
        let ex1 = ["/* blah blah blah\n      foooo baaar\n   blah */"];
        assert_eq!(
            cleanup_comments(&mut ex1.iter()),
            "blah blah blah\n   foooo baaar\nblah"
        );

        let ex2 = ["# a1", "#    a2", "# aa"];
        assert_eq!(cleanup_comments(&mut ex2.iter()), "aa\n   a2\na1");
    }

    #[test]
    fn test_dedent() {
        let ex1 = "a\n   b\n   c\n     d";
        assert_eq!(dedent_comment(ex1), "a\nb\nc\n  d");
        let ex2 = "a\nb\nc";
        assert_eq!(dedent_comment(ex2), ex2);
        let ex3 = "   a\n   b\n\n     c";
        assert_eq!(dedent_comment(ex3), "a\nb\n\n  c");
    }

    #[test]
    fn test_single_line_comment_stripping() {
        let ex1 = "    * a";
        let ex2 = "    # a";
        let ex3 = "   a";
        let ex4 = "   *";
        assert_eq!(cleanup_single_line(ex1), " a");
        assert_eq!(cleanup_single_line(ex2), " a");
        assert_eq!(cleanup_single_line(ex3), ex3);
        assert_eq!(cleanup_single_line(ex4), "");
    }

    #[test]
    fn test_single_line_retains_bold_headings() {
        let ex1 = "   **Foo**:";
        assert_eq!(cleanup_single_line(ex1), ex1);
    }

    #[test]
    fn test_regression_11() {
        let out = r#"Create a fixed width string with additional prefix to match
required width.

This function will fail if the input string is longer than the
requested length.

Type: fixedWidthString :: int -> string -> string

Example:
  fixedWidthString 5 "0" (toString 15)
  => "00015""#;
        let ast = rnix::parse(include_str!("../testdata/regression-11.nix"))
            .as_result()
            .unwrap();
        let results = search_ast(&regex::Regex::new("fixedWidthString").unwrap(), &ast);
        assert_eq!(results.len(), 1);

        assert_eq!(results[0].doc, out);
    }
}
