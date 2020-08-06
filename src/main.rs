//! A nix documentation search program
use nix_doc::pprint::pprint_args;
use nix_doc::threadpool::ThreadPool;

use colorful::{Color, Colorful};
use regex::Regex;
use rnix::types::{AttrSet, EntryHolder, Ident, Lambda, TokenWrapper, TypedNode};
use rnix::SyntaxKind::*;
use rnix::{NodeOrToken, SyntaxNode, WalkEvent, AST};
use walkdir::WalkDir;

use std::env;
use std::fs;
use std::iter;
use std::path::Path;
use std::sync::mpsc::channel;
use std::{fmt::Display, str};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
fn is_searchable(fname: &Path) -> bool {
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

/// Search the `dir` for files with function definitions matching `matching`
fn search<F>(dir: &Path, matching: Regex, should_search: F)
where
    F: Fn(&Path) -> bool,
{
    let pool = ThreadPool::new();
    let (tx, rx) = channel();

    //println!("searching {}", dir.display());
    for direntry in WalkDir::new(dir)
        .into_iter()
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
            if formatted.len() > 0 {
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
                if let Some(set) = enter.into_node().and_then(|elem| AttrSet::cast(elem)) {
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
                    .split("\n")
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

                // we now know it is a function we are looking for
                // grab the arguments
                let param_block = pprint_args(&lambda);

                // find the doc comment
                if let Some(comment) = find_comment(attr.node().clone()) {
                    results.push(SearchResult {
                        identifier: ident_name.to_string(),
                        doc: comment,
                        param_block,
                        defined_at_start: defined_at_start.unwrap(),
                    });
                } else {
                    // ignore results without comments, they are probably reexports or
                    // wrappers
                    continue;
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
            t if t.is_trivia() => (),
            _ => break,
        }
    }
    let doc = cleanup_comments(&mut comments.iter().map(|c| c.as_str()));
    return Some(doc).filter(|it| !it.is_empty());
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let re_match = args.next();
    let file = args.next().unwrap_or(".".to_string());
    if re_match.is_none() {
        eprintln!("Usage: nix-doc SearchRegex [Directory]");
        return Ok(());
    }

    let re_match = re_match.unwrap();
    let re_match = Regex::new(&re_match)?;
    search(&Path::new(&file), re_match, is_searchable);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_stripping() {
        let ex1 = ["/* blah blah blah\n      foooo baaar\n */"];
        assert_eq!(
            cleanup_comments(&mut ex1.iter()),
            "blah blah blah\nfoooo baaar"
        );

        let ex2 = ["# a1", "#    a2", "# aa"];
        assert_eq!(cleanup_comments(&mut ex2.iter()), "aa\na2\na1");
    }
}
