// SPDX-FileCopyrightText: 2024 Jade Lovelace
//
// SPDX-License-Identifier: BSD-2-Clause OR MIT

//! A nix documentation search program

use nix_doc::{is_searchable, search, tags, Result};

use regex::Regex;
use structopt::StructOpt;

use std::{fs, io::BufWriter, path::PathBuf};

#[derive(StructOpt, Debug)]
#[structopt(about = "an AST based Nix documentation tool")]
enum Args {
    /// Search a directory of nix files for the given function
    Search {
        /// Regex to search with
        re: String,

        /// Directory to search
        #[structopt(default_value = ".")]
        dir: PathBuf,
    },

    /// Generates a ctags compatible database for a directory of nix files
    Tags {
        /// The directory
        #[structopt(default_value = ".")]
        dir: PathBuf,

        /// The maximum number of times to emit the same tag name. Avoids tags
        /// such as "version" appearing uselessly a very large number of times.
        ///
        /// Pass -1 to disable this optimization.
        #[structopt(long, default_value = "500")]
        max_cardinality: i32,
    },
}

fn main() -> Result<()> {
    let args = Args::from_args();

    match args {
        Args::Search { re, dir } => {
            let re_match = Regex::new(&re)?;
            search(&dir, re_match, is_searchable);
        }

        Args::Tags {
            dir,
            max_cardinality,
        } => {
            let h = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open("tags")?;
            let mut h = BufWriter::new(h);

            let res = tags::run_on_dir(
                &dir,
                if max_cardinality >= 0 {
                    Some(max_cardinality as u32)
                } else {
                    None
                },
                &mut h,
            );
            if let Err(e) = res {
                eprintln!("Failed while ctags'ing: {:?}", e);
            }
        }
    }
    Ok(())
}
