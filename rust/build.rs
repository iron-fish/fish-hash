use cmake::Config;

fn main() {
    println!("cargo:rerun-if-changed=../cpp");

    // Build static library
    let dst = Config::new("../cpp").very_verbose(true).build();
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=FishHash");
    let target = std::env::var("TARGET").unwrap();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else {
        unimplemented!();
    }
}
