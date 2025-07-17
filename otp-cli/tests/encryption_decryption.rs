use std::fs;
use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_full_vault_workflow() {
    // 1. Setup temporary directories for the test
    let temp_dir = tempdir().unwrap();
    let vault_path = temp_dir.path().join("my_test_vault");
    let input_path = temp_dir.path().join("input.txt");
    let encrypted_path = temp_dir.path().join("encrypted.bin");
    let decrypted_path = temp_dir.path().join("decrypted.txt");
    let metadata_path = temp_dir.path().join("encrypted.bin.metadata.json");

    let input_content = "This is a new test for the vault-based OTP encryption system.";
    fs::write(&input_path, input_content).unwrap();

    // 2. Initialize the vault
    let mut cmd_init = Command::cargo_bin("otp-cli").unwrap();
    cmd_init
        .arg("--vault")
        .arg(&vault_path)
        .arg("vault")
        .arg("init")
        .assert()
        .success();
    assert!(vault_path.join("vault_state.json").exists());
    assert!(vault_path.join("pads/available").exists());

    // 3. Generate a pad and capture its ID
    let mut cmd_gen = Command::cargo_bin("otp-cli").unwrap();
    let generate_output = cmd_gen
        .arg("--vault")
        .arg(&vault_path)
        .arg("pad")
        .arg("generate")
        .arg("--size")
        .arg("1") // 1 MB
        .output()
        .expect("Failed to execute pad generate");
    
    assert!(generate_output.status.success());
    let pad_id = String::from_utf8(generate_output.stdout).unwrap().trim().to_string();
    assert!(!pad_id.is_empty(), "Pad ID should not be empty");

    // 4. Encrypt the file using the new pad
    let mut cmd_encrypt = Command::cargo_bin("otp-cli").unwrap();
    cmd_encrypt
        .arg("--vault")
        .arg(&vault_path)
        .arg("encrypt")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&encrypted_path)
        .arg("--pad-id")
        .arg(&pad_id)
        .assert()
        .success();
    
    assert!(encrypted_path.exists(), "Encrypted file should exist");
    assert!(metadata_path.exists(), "Metadata file should exist");

    // 5. Decrypt the file
    let mut cmd_decrypt = Command::cargo_bin("otp-cli").unwrap();
    cmd_decrypt
        .arg("--vault")
        .arg(&vault_path)
        .arg("decrypt")
        .arg("--input")
        .arg(&encrypted_path)
        .arg("--output")
        .arg(&decrypted_path)
        .arg("--metadata")
        .arg(&metadata_path)
        .assert()
        .success();

    // 6. Verify the decrypted content
    let decrypted_content = fs::read_to_string(&decrypted_path).unwrap();
    assert_eq!(input_content, decrypted_content);
}

#[test]
fn test_vault_status_command() {
    // 1. Setup temporary directories for the test
    let temp_dir = tempdir().unwrap();
    let vault_path = temp_dir.path().join("my_test_vault");

    // 2. Initialize the vault
    let mut cmd_init = Command::cargo_bin("otp-cli").unwrap();
    cmd_init
        .arg("--vault")
        .arg(&vault_path)
        .arg("vault")
        .arg("init")
        .assert()
        .success();

    // 3. Generate a few pads
    for _ in 0..3 {
        let mut cmd_gen = Command::cargo_bin("otp-cli").unwrap();
        cmd_gen
            .arg("--vault")
            .arg(&vault_path)
            .arg("pad")
            .arg("generate")
            .arg("--size")
            .arg("1") // 1 MB
            .assert()
            .success();
    }

    // 4. Run `vault status` and check the output
    let mut cmd_status = Command::cargo_bin("otp-cli").unwrap();
    cmd_status
        .arg("--vault")
        .arg(&vault_path)
        .arg("vault")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Pads: 3"))
        .stdout(predicate::str::contains("Available: 3"))
        .stdout(predicate::str::contains("Fully Used: 0"))
        .stdout(predicate::str::contains("Total Storage: 3.00 MB"));
}

#[test]
fn test_pad_delete_command() {
    // 1. Setup
    let temp_dir = tempdir().unwrap();
    let vault_path = temp_dir.path().join("my_test_vault");
    Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("init")
        .assert().success();

    // 2. Generate a pad
    let generate_output = Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("generate")
        .output().unwrap();
    let pad_id = String::from_utf8(generate_output.stdout).unwrap().trim().to_string();

    // 3. Delete the pad
    Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("delete").arg("--pad-id").arg(&pad_id)
        .assert().success();

    // 4. Verify deletion
    Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("list")
        .assert().success()
        .stdout(predicate::str::contains("No pads found"));
}

#[test]
fn test_encryption_decryption_user_flow() {
    // 1. Setup
    let temp_dir = tempdir().unwrap();
    let vault_path = temp_dir.path().join("my_test_vault");
    let input_content = "This is a full user flow test for encryption and decryption.";
    let input_path = temp_dir.path().join("user_flow_input.txt");
    fs::write(&input_path, input_content).unwrap();

    // 2. Initialize vault
    Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("init")
        .assert().success();

    // 3. Generate a pad
    let generate_output = Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("generate")
        .output().unwrap();
    let pad_id = String::from_utf8(generate_output.stdout).unwrap().trim().to_string();

    // 4. Encrypt
    let encrypted_path = temp_dir.path().join("user_flow_encrypted.bin");
    Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("encrypt")
        .arg("--input").arg(&input_path)
        .arg("--output").arg(&encrypted_path)
        .arg("--pad-id").arg(&pad_id)
        .assert().success();

    // 5. Decrypt
    let decrypted_path = temp_dir.path().join("user_flow_decrypted.txt");
    let metadata_path = temp_dir.path().join("user_flow_encrypted.bin.metadata.json");
    Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("decrypt")
        .arg("--input").arg(&encrypted_path)
        .arg("--output").arg(&decrypted_path)
        .arg("--metadata").arg(&metadata_path)
        .assert().success();

    // 6. Verify
    let decrypted_content = fs::read_to_string(&decrypted_path).unwrap();
    assert_eq!(input_content, decrypted_content);
}

#[test]
fn test_pad_list_command() {
    // 1. Setup
    let temp_dir = tempdir().unwrap();
    let vault_path = temp_dir.path().join("my_test_vault");
    Command::cargo_bin("otp-cli").unwrap()
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("init")
        .assert().success();

    // 2. Generate two pads
    let mut pad_ids = Vec::new();
    for _ in 0..2 {
        let generate_output = Command::cargo_bin("otp-cli").unwrap()
            .arg("--vault").arg(&vault_path)
            .arg("pad").arg("generate")
            .output().unwrap();
        let pad_id = String::from_utf8(generate_output.stdout).unwrap().trim().to_string();
        pad_ids.push(pad_id);
    }

    // 3. List pads and verify
    let mut cmd_list = Command::cargo_bin("otp-cli").unwrap();
    cmd_list
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("list")
        .assert().success()
        .stdout(predicate::str::contains(&pad_ids[0]))
        .stdout(predicate::str::contains(&pad_ids[1]));
}