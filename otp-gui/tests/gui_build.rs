use std::process::Command;

#[test]
fn test_gui_builds_successfully() {
    let mut cmd = Command::new("cargo");
    cmd.arg("tauri")
        .arg("build")
        .current_dir(".."); // Run from the otp-gui directory

    let output = cmd.output().expect("Failed to execute command");

    assert!(output.status.success(), "GUI build failed: {}", String::from_utf8_lossy(&output.stderr));
}