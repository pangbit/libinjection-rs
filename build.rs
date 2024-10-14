use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUILD_DIR_NAME: &str = "libinjection";

fn run(cmd: &str, args: &[&str], cwd: &Path) -> bool {
    let output = Command::new(cmd)
        .args(args)
        .env("OUT_DIR", env::var("OUT_DIR").unwrap())
        .current_dir(cwd)
        .output()
        .unwrap();
    if output.status.success() {
        true
    } else {
        panic!("make error: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_parent_dir = out_path.join(BUILD_DIR_NAME);

    if !run(
        "cp",
        &["-r", "src/libinjection", build_parent_dir.to_str().unwrap()],
        Path::new("."),
    ) {
        panic!("unable to copy libinjection");
    }

    if !run("bash", &["autogen.sh"], build_parent_dir.as_path()) {
        panic!("unable to run autogen.sh");
    }

    if !run("bash", &["configure"], build_parent_dir.as_path()) {
        panic!("unable to run configure");
    }

    if !run("make", &["-C", "src"], build_parent_dir.as_path()) {
        panic!("unable to make libinjection");
    }

    if !run(
        "ar",
        &[
            "-crs",
            "libinjection.a",
            "libinjection_sqli.o",
            "libinjection_html5.o",
            "libinjection_xss.o",
        ],
        build_parent_dir.join("src").as_path(),
    ) {
        panic!("unable to build static library");
    }

    println!("cargo:rustc-link-lib=static=injection");
    println!(
        "cargo:rustc-link-search={}",
        build_parent_dir.join("src").display()
    );

    let h_path = build_parent_dir.join("src/libinjection.h");
    let bindings = bindgen::Builder::default()
        .header(h_path.to_str().unwrap())
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("unable to write bindings");
}
