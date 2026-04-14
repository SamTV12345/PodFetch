fn main() {
    println!("cargo:rerun-if-changed=../../migrations/sqlite");
    println!("cargo:rerun-if-changed=../../migrations/postgres");
    println!("cargo:rerun-if-changed=../../migrations/mysql");
}
