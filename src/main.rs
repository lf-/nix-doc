//! A nix documentation search program

use nix_doc::{is_searchable, search, Result};

use regex::Regex;

use std::env;
use std::path::Path;

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
