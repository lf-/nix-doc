use std::env;
use std::fs;
use std::str;

use rnix::types::{Lambda, TypedNode};
use rnix::NodeOrToken;
use rnix::SyntaxKind::*;

use nix_doc::pprint;

fn main() {
    let mut args = env::args().skip(1).take(2);

    let op = args.next().expect("missing op argument");
    let file = args.next().expect("missing file argument");
    let file = fs::read(file).unwrap();

    let parsed = rnix::parse(str::from_utf8(&file).unwrap());

    match op.as_str() {
        "dump" => {
            for node in parsed.node().descendants_with_tokens() {
                match node {
                    NodeOrToken::Node(n) => {
                        println!("N {:?} {}", n.kind(), n);
                    }
                    NodeOrToken::Token(t) => {
                        println!("T {:?} {}", t.kind(), t);
                    }
                }
            }
        }

        "pprint" => {
            // tests pprint on all the functions it finds
            for node in parsed.node().descendants() {
                match node.kind() {
                    NODE_LAMBDA => {
                        println!("lambda!! {}", node);
                        let lambda = Lambda::cast(node).unwrap();
                        println!("pprint_args: {}", pprint::pprint_args(&lambda));
                        let arg = lambda.arg().unwrap();
                        println!("args!! {:?} {}", arg.kind(), arg);
                    }
                    _ => {}
                }
            }
        }

        _ => {
            panic!("unknown operation, supported: dump, pprint");
        }
    }
}
