use std::process::Command;

fn main() {
    // disable compilation now
    // if true { return }

    println!("cargo:rerun-if-changed=edb-src");
    let res = Command::new("../third-party/EdenDB/target/release/edendb")
        .env("EDB_EXPOSE_DESER", "t")
        .arg("main.edl")
        .arg("test_data.edl")
        .arg("--rust-output-directory")
        .arg("../src/")
        .arg("--dump-source-file")
        .arg("../src/bin/source_schema.bin")
        .current_dir("edb-src")
        .status()
        .unwrap();

    assert!(res.success());
}
