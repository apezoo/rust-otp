use std::fs;
use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_gui_workflow_commands() {
    // 1. Setup
    let temp_dir = tempdir().unwrap();
    let vault_path = temp_dir.path().join("my_gui_vault");
    let input_content = "GUI workflow test.";
    let input_path = temp_dir.path().join("gui_input.txt");
    fs::write(&input_path, input_content).unwrap();

    let otp_cli_path = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("otp-cli/target/debug/otp-cli");

    // 2. Initialize Vault
    Command::new(&otp_cli_path)
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("init")
        .assert().success();

    // 3. Generate a Pad
    let generate_output = Command::new(&otp_cli_path)
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("generate")
        .output().unwrap();
    let pad_id = String::from_utf8(generate_output.stdout).unwrap().trim().to_string();

    // 4. List Pads to ensure it was created
    Command::new(&otp_cli_path)
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("list")
        .assert().success()
        .stdout(predicate::str::contains(&pad_id));

    // 5. Encrypt the file
    let encrypted_path = temp_dir.path().join("gui_encrypted.bin");
    Command::new(&otp_cli_path)
        .arg("--vault").arg(&vault_path)
        .arg("encrypt")
        .arg("--input").arg(&input_path)
        .arg("--output").arg(&encrypted_path)
        .arg("--pad-id").arg(&pad_id)
        .assert().success();

    // 6. Check Vault Status to see usage
    Command::new(&otp_cli_path)
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("status")
        .assert().success()
        .stdout(predicate::str::contains("Used:"));

    // 7. Decrypt the file
    let decrypted_path = temp_dir.path().join("gui_decrypted.txt");
    let metadata_path = temp_dir.path().join("gui_encrypted.bin.metadata.json");
    Command::new(&otp_cli_path)
        .arg("--vault").arg(&vault_path)
        .arg("decrypt")
        .arg("--input").arg(&encrypted_path)
        .arg("--output").arg(&decrypted_path)
        .arg("--metadata").arg(&metadata_path)
        .assert().success();
    let decrypted_content = fs::read_to_string(&decrypted_path).unwrap();
    assert_eq!(input_content, decrypted_content);

    // 8. Delete the pad
    Command::new(&otp_cli_path)
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("delete").arg("--pad-id").arg(&pad_id)
        .assert().success();

    // 9. List pads to confirm deletion
    Command::new(&otp_cli_path)
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("list")
        .assert().success()
        .stdout(predicate::str::contains("No pads found"));
}