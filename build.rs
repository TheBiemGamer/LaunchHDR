fn main() {
    let now = chrono::Local::now();
    println!("cargo:rustc-env=COMPILE_DATE={}", now.format("%Y-%m-%d"));
}
