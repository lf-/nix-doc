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

    cc::Build::new()
        .cpp(true)
        .opt_level(2)
        .shared_flag(true)
        .flag("-std=c++17")
        .add_pkg_config(nix_expr)
        .add_pkg_config(nix_store)
        .add_pkg_config(nix_main)
        .file("plugin.cpp")
        .compile("nix_doc_plugin.so");
}
