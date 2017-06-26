extern crate gcc;

fn main() {
    gcc::compile_library("libxpm.a", &["src/xpm.c"]);
}
