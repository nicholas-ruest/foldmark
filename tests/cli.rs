use std::process::Command;

#[test]
fn cli_emits_json_plan() {
    let output = Command::new(env!("CARGO_BIN_EXE_foldmark"))
        .args(["6", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["document_pages"], 6);
    assert_eq!(json["total_sheets"], 2);
    assert_eq!(json["total_blank_positions"], 2);
}

#[test]
fn cli_rejects_zero_pages() {
    let output = Command::new(env!("CARGO_BIN_EXE_foldmark"))
        .arg("0")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("0 is not in 1.."));
}

#[test]
fn cli_writes_csv_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("plan.csv");
    let output = Command::new(env!("CARGO_BIN_EXE_foldmark"))
        .args(["8", "--format", "csv", "--output"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let csv = std::fs::read_to_string(path).unwrap();
    assert!(csv.contains("1,2,back,4,5"));
}
