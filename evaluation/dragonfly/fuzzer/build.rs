
fn main() {
    cc::Build::new()
        .file("generator.c")
        .include(".")
        .define("DISABLE_random_buffer", "")
        .warnings(true)
        .extra_warnings(true)
        .debug(true)
        .compile("generator");
}
