//! library components of nix-doc
pub mod pprint;
pub mod threadpool;

use crate::pprint::pprint_args;
use crate::threadpool::ThreadPool;

use colorful::{Color, Colorful};
use regex::Regex;
use rnix::types::{AttrSet, EntryHolder, Ident, Lambda, TokenWrapper, TypedNode};
use rnix::SyntaxKind::*;
use rnix::{NodeOrToken, SyntaxNode, TextUnit, WalkEvent, AST};
use walkdir::{DirEntry, WalkDir};

use std::ffi::{CStr, CString};
use std::fs;
use std::iter;
use std::os::raw::c_char;
use std::panic;
use std::path::Path;
use std::ptr;
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
fn is_ignored(entry: &DirEntry) -> bool {
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

/// Deletes whitespace and leading comment characters
///
/// Oversight we are choosing to ignore: if you put # characters at the beginning of lines in a
/// multiline comment, they will be deleted.
fn cleanup_comments<S: AsRef<str>, I: DoubleEndedIterator<Item = S>>(comment: &mut I) -> String {
    textwrap::dedent(
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
                    .map(|line| {
                        // leading whitespace + single line comments
                        line.trim_start_matches(|c: char| c.is_whitespace())
                            .trim_start_matches(|c: char| c == '#' || c == '*')
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// Get the docs for a function in the given file path at the given file position and return it as
/// a C string pointer
#[no_mangle]
extern "C" fn nd_get_function_docs(
    filename: *const c_char,
    line: usize,
    col: usize,
) -> *const c_char {
    let fname = unsafe { CStr::from_ptr(filename) };
    fname
        .to_str()
        .ok()
        .and_then(|f| {
            panic::catch_unwind(|| get_function_docs(f, line, col))
                .map_err(|e| {
                    eprintln!("panic!! {:#?}", e);
                    e
                })
                .ok()
        })
        .flatten()
        .and_then(|s| CString::new(s).ok())
        .map(|s| s.into_raw() as *const c_char)
        .unwrap_or(ptr::null())
}

/// Call this to free a string from nd_get_function_docs
#[no_mangle]
extern "C" fn nd_free_string(s: *const c_char) {
    unsafe {
        // this is maybe UB, but it is immediately dropped.
        CString::from_raw(s as *mut c_char);
    }
}

/// Get the docs for a specific function
fn get_function_docs(filename: &str, line: usize, col: usize) -> Option<String> {
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
            NODE_IDENT | NODE_KEY | NODE_KEY_VALUE | TOKEN_IDENT | TOKEN_ASSIGN => (),
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
        let ex1 = ["/* blah blah blah\n      foooo baaar\n */"];
        assert_eq!(
            cleanup_comments(&mut ex1.iter()),
            "blah blah blah\nfoooo baaar"
        );

        let ex2 = ["# a1", "#    a2", "# aa"];
        assert_eq!(cleanup_comments(&mut ex2.iter()), "aa\n   a2\na1");
    }
}
