use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn fixture_contents(relative_path: &str) -> String {
    let path = fixture_path(relative_path);
    fs::read_to_string(path).expect("read fixture")
}

pub fn run_fixture(agent: &str, relative_path: &str) -> Output {
    let stdin_json = fixture_contents(relative_path);
    let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
    let stdin_bytes = stdin_json.as_bytes();
    tmp.write_all(stdin_bytes).expect("write temp file");
    let tmp_path = tmp.into_temp_path();

    let stdin_file = std::fs::File::open(tmp_path).expect("open temp file");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Command::new(env!("CARGO_BIN_EXE_agent-tool-check"))
        .arg(agent)
        .current_dir(manifest_dir)
        .stdin(stdin_file)
        .output()
        .expect("run binary")
}

fn fixture_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative_path)
}
