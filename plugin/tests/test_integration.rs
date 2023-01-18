use std::{env, path::PathBuf, process::Command};

use expect_test::{expect, Expect};

fn get_target_path() -> PathBuf {
    env::var_os("CARGO_BIN_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe().ok().map(|mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            })
        })
        .unwrap_or_else(|| panic!("CARGO_BIN_PATH wasn't set. Cannot continue running test"))
}

fn get_plugin_path() -> PathBuf {
    let name = if cfg!(target_os = "linux") {
        "libnix_doc_plugin.so"
    } else if cfg!(target_os = "macos") {
        "libnix_doc_plugin.dylib"
    } else {
        unimplemented!("non linux or macos platform")
    };
    let mut p = get_target_path();
    p.push(name);
    p
}

fn nix_eval(expr: &str) -> String {
    let temp_state_dir = tempfile::tempdir().expect("making tempdir");
    println!("PATH: {}", env::var("PATH").unwrap());
    let output = Command::new("nix")
        .env("NIX_STATE_DIR", temp_state_dir.path())
        .args(["--extra-experimental-features", "nix-command flakes"])
        .arg("--plugin-files")
        .arg(get_plugin_path())
        .args(["eval", "--raw", "--impure"])
        .arg("--expr")
        .arg(expr)
        .output()
        .expect("failed calling nix eval");
    if !output.status.success() {
        panic!(
            "Error running nix eval:\nstderr:\n{}\nstdout:\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }
    String::from_utf8(output.stdout).unwrap()
}

fn sanitize_paths(value: &str) -> String {
    value
        .split(' ')
        .map(|word| {
            if word.contains("testdata") {
                let last_slash = word.rfind('/');
                match last_slash {
                    Some(pos) => &word[pos + 1..],
                    None => word,
                }
            } else {
                word
            }
        })
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

#[test]
fn sanitize_paths_works() {
    assert_eq!(
        sanitize_paths("/src/nix-doc/testdata/test.nix:5"),
        "test.nix:5"
    );
}

fn check(fixture: &str, expected: Expect) {
    let stripped = strip_ansi_escapes::strip(sanitize_paths(&nix_eval(fixture))).unwrap();
    let output = String::from_utf8_lossy(&stripped);
    expected.assert_eq(&output);
}

#[test]
fn test_builtins_doc() {
    check(
        r#"builtins.seq (builtins.doc (import testdata/test.nix).addOne) """#,
        expect![[r#"
               line one
               line two
               line three
            func = x: ...
            # test.nix:5
        "#]],
    );

    check(
        r#"builtins.seq (builtins.doc (import ./testdata/test.nix).the-snd-fn) """#,
        expect![[r#"
               this one
               has multiple
               comments
            func = { b, /* doc */ c }: ...
            # test.nix:10
        "#]],
    );
}

#[test]
fn test_builtins_getdoc() {
    check(
        r#"builtins.getDoc (import ./testdata/test.nix).the-snd-fn"#,
        expect![[r#"
               this one
               has multiple
               comments
            func = { b, /* doc */ c }: ...
            # test.nix:10"#]],
    );
}
