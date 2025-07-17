#![allow(missing_docs)]
fn main() {
    static_files::resource_dir("../static").build().unwrap();
}