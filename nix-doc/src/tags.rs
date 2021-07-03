use std::env::current_dir;
use std::io::Write;
use std::sync::mpsc::channel;
use std::{
    fmt, fs, io,
    iter::FromIterator,
    path::{Path, PathBuf},
};

use rnix::{
    types::{AttrSet, EntryHolder, Ident, TokenWrapper, TypedNode},
    SmolStr,
    SyntaxKind::*,
    AST,
};
use walkdir::WalkDir;

use crate::threadpool::ThreadPool;
use crate::{is_ignored, is_searchable};

enum Kind {
    Function,
    Member,
}

/// Path interned in an array of all the paths.
#[derive(Clone, Copy, Debug)]
struct InternedPath(usize);

macro_rules! impl_from {
    ($on:ty, $variant:ident, $ty:ty) => {
        impl From<$ty> for $on {
            fn from(f: $ty) -> $on {
                <$on>::$variant(f)
            }
        }
    };
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl_from!(Error, Io, io::Error);

/// One ctags file entry
struct Tag {
    /// Name of the identifier
    name: SmolStr,

    /// Path relative to the tags file parent dir
    path: InternedPath,

    /// "address" of the tag, the line it's on, basically.
    addr: SmolStr,

    /// Kind of tag
    kind: Kind,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Function => write!(f, "f"),
            Kind::Member => write!(f, "m"),
        }
    }
}

fn make_addr(a: &str) -> SmolStr {
    // FIXME: delete this cloned malarkey when we can tell everyone with old nixpkgs to go eat a
    // nixpkgs-unstable cookie
    SmolStr::from_iter(["/^", &a.replace(r"\", r"\\"), "$/"].iter().cloned())
}

impl Tag {
    fn to_string_relative_to(&self, paths: &[PathBuf], p: &Path) -> Option<String> {
        let relpath = pathdiff::diff_paths(&paths[self.path.0], p)?;
        Some(format!(
            "{}\t{}\t{};\"\t{}",
            self.name,
            relpath.display(),
            make_addr(&self.addr),
            self.kind
        ))
    }
}

struct FileJob<'a> {
    file: InternedPath,
    source: &'a str,
    results: &'a mut Vec<Tag>,
}

impl<'a> FileJob<'a> {
    fn visit_attrset(&mut self, set: &AttrSet) {
        for ent in set.entries() {
            let tag = (|| {
                let val = ent.value()?;
                let key = ent.key()?;

                let kind = match val.kind() {
                    NODE_LAMBDA => Kind::Function,
                    _ => Kind::Member,
                };

                let defined_at_start = key.node().text_range().start().to_usize();
                let prior = &self.source[..defined_at_start];
                let line_start = prior.rfind('\n').unwrap_or(0);
                let after = &self.source[defined_at_start..];
                let line_end = after
                    .find('\n')
                    .unwrap_or(self.source.len() - defined_at_start);
                let source_line = &self.source[line_start..defined_at_start + line_end];
                let source_line = source_line.strip_prefix('\n').unwrap_or(source_line);

                let ident = key.path().last().and_then(Ident::cast);
                let ident_name = ident.as_ref().map(|id| id.as_str())?;

                Some(Tag {
                    name: ident_name.into(),
                    path: self.file.clone(),
                    addr: source_line.into(),
                    kind,
                })
            })();

            if let Some(tag) = tag {
                self.results.push(tag);
            }
        }
    }

    fn exec(&mut self, ast: &AST) {
        for evt in ast.node().preorder_with_tokens() {
            match evt {
                rnix::WalkEvent::Enter(ent) => {
                    if let Some(set) = ent.into_node().and_then(AttrSet::cast) {
                        self.visit_attrset(&set);
                    }
                }
                rnix::WalkEvent::Leave(_) => (),
            }
        }
    }

    /// Runs a file job collecting tags for a path.
    ///
    /// `p` must be absolute.
    pub fn run(p_interned: InternedPath, p: &Path) -> Result<Vec<Tag>, Error> {
        assert!(p.is_absolute());
        let contents = fs::read_to_string(p)?;
        let parsed = rnix::parse(&contents);
        let mut results = Vec::new();

        let mut job = FileJob {
            file: p_interned,
            source: &contents,
            results: &mut results,
        };

        job.exec(&parsed);

        // we sort here because the rust sorting algo is supposedly good at a bunch of concatenated
        // sorted lists, and parallel compute is effectively free
        results.sort_unstable_by(|e1, e2| e1.name.as_str().cmp(e2.name.as_str()));

        Ok(results)
    }
}

/// Builds a tags database into the given writer with paths relative to the current directory, with
/// the nix files in `dir`
pub fn run_on_dir(dir: &Path, mut writer: impl Write) -> Result<(), Error> {
    let pool = ThreadPool::default();
    let (tx, rx) = channel();

    let mut paths_interned = Vec::new();
    let curdir = current_dir()?;

    //println!("searching {}", dir.display());
    for direntry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
        .filter_map(|e| e.ok())
        .filter(|e| is_searchable(e.path()) && e.path().is_file())
    {
        let path = curdir.join(direntry.into_path());
        let path_ = path.clone();
        paths_interned.push(path);
        let path_interned = InternedPath(paths_interned.len() - 1);

        let my_tx = tx.clone();
        pool.push(move || {
            let results = FileJob::run(path_interned, &path_);
            let results = match results {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error processing {}: {:?}", &path_.display(), e);
                    return;
                }
            };

            if !results.is_empty() {
                my_tx.send(results).expect("failed to send tags");
            }
        });
    }

    drop(tx);
    pool.done();

    let mut out = Vec::new();
    while let Ok(set) = rx.recv() {
        out.extend(set);
    }

    out.sort_by(|e1, e2| e1.name.as_str().cmp(e2.name.as_str()));

    for tag in out {
        let tag = match tag.to_string_relative_to(&paths_interned, &curdir) {
            Some(v) => v,
            None => continue,
        };
        writer.write(tag.as_bytes())?;
        writer.write(b"\n")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        env::{current_dir, set_current_dir},
        path::PathBuf,
    };

    use super::*;

    #[test]
    fn smoke() {
        let curdir = current_dir().unwrap();

        let datadir = curdir.join("testdata");
        println!("datadir: {}", &datadir.display());
        set_current_dir(datadir).unwrap();
        let mut out = Vec::new();

        run_on_dir(&PathBuf::from("."), &mut out).unwrap();
        let out_s = std::str::from_utf8(&out).unwrap();
        println!("{}", out_s);

        assert_eq!(
            out_s.trim(),
            r#"
c	test.nix	/^   a.b.c = a: 1;$/;"	f
fixedWidthString	regression-11.nix	/^  fixedWidthString = width: filler: str:$/;"	f
the-fn	test.nix	/^   the-fn = a: b: {z = a; y = b;};$/;"	f
the-snd-fn	test.nix	/^   the-snd-fn = {b, /* doc */ c}: {};$/;"	f
withFeature	regression-11.nix	/^  withFeature = with_: feat: "--${if with_ then "with" else "without"}-${feat}";$/;"	f
withFeatureAs	regression-11.nix	/^  withFeatureAs = with_: feat: value: withFeature with_ feat + optionalString with_ "=${value}";$/;"	f
y	test.nix	/^   the-fn = a: b: {z = a; y = b;};$/;"	m
z	test.nix	/^   the-fn = a: b: {z = a; y = b;};$/;"	m
"#.trim()
        );

        set_current_dir(curdir).unwrap();
    }
}
