fn main() {
    cc::Build::new().cpp(true).file("dec.cpp").compile("dec.a");
    println!("Build!")
}
