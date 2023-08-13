trait AddPkg {
    fn add_pkg_config(&mut self, pkg: pkg_config::Library) -> &mut Self;
}
impl AddPkg for cc::Build {
    fn add_pkg_config(&mut self, pkg: pkg_config::Library) -> &mut Self {
        for p in pkg.include_paths.into_iter() {
            self.flag("-isystem").flag(p.to_str().unwrap());
        }
        for p in pkg.link_paths.into_iter() {
            self.flag(&format!("-L{:?}", p));
        }
        for p in pkg.libs.into_iter() {
            self.flag(&format!("-l{}", p));
        }
        for p in pkg.framework_paths.into_iter() {
            self.flag(&format!("-F{:?}", p));
        }
        for p in pkg.frameworks.into_iter() {
            self.flag(&format!("-framework {}", p));
        }
        self
    }
}

fn main() {
    #[cfg(test)]
    {
        return;
    }

    println!("cargo:rerun-if-changed=plugin.cpp");

    let nix_expr = pkg_config::Config::new()
        .atleast_version("2.1.1")
        .probe("nix-expr")
        .unwrap();
    let nix_store = pkg_config::Config::new()
        .atleast_version("2.1.1")
        .probe("nix-store")
        .unwrap();
    let nix_main = pkg_config::Config::new()
        .atleast_version("2.1.1")
        .probe("nix-main")
        .unwrap();

    let nix_ver = nix_expr.version.clone();

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .opt_level(2)
        .shared_flag(true)
        .flag("-std=c++20")
        .add_pkg_config(nix_expr)
        .add_pkg_config(nix_store)
        .add_pkg_config(nix_main)
        .define("BUILD_NIX_VERSION", Some(nix_ver.as_str()))
        .cargo_metadata(true)
        .link_lib_modifier("+whole-archive")
        .file("plugin.cpp");

    let mut parts = nix_ver.split('.').map(str::parse);
    let major: u32 = parts.next().unwrap().unwrap();
    let minor = parts.next().unwrap().unwrap();
    let patch = parts.next().map(|x| x.ok()).flatten().unwrap_or(0);
    println!("Nix version: major={major} minor={minor} patch={patch}");

    // Indicate that we need to patch around an API change with macros
    if (major, minor) >= (2, 4) {
        build.define("NIX_2_4_0", None);
    }
    if (major, minor) >= (2, 6) {
        build.define("NIX_2_6_0", None);
    }
    if (major, minor) >= (2, 9) {
        build.define("NIX_2_9_0", None);
    }
    if (major, minor) >= (2, 13) {
        build.define("NIX_2_13_0", None);
    }
    if (major, minor, patch) >= (2, 13, 1) {
        build.define("NIX_2_13_1", None);
    }
    if (major, minor) >= (2, 14) {
        build.define("NIX_2_14_0", None);
    }
    if (major, minor) >= (2, 16) {
        build.define("NIX_2_16_0", None);
    }
    if (major, minor) >= (2, 17) {
        build.define("NIX_2_17_0", None);
    }

    build.compile("nix_doc_plugin");
}
