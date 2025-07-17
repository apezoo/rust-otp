use std::fs;
use std::process::Command;
use assert_cmd::prelude::*;
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