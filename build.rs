extern crate cc;

fn main() {
    cc::Build::new()
        .cpp(true)
        .file("compression.cc")
        .flag("--std=c++14")
        .compile("libcompression.a");

    println!("cargo:rustc-link-lib=draco");
}
