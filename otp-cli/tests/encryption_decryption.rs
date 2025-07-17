use std::fs;
use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_encrypt_decrypt_flow() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.txt");
    let encrypted_path = temp_dir.path().join("encrypted.bin");
    let decrypted_path = temp_dir.path().join("decrypted.txt");
    let metadata_path = temp_dir.path().join("encrypted.bin.metadata.json");
    let pad_path = temp_dir.path().join("test.pad");

    let input_content = "This is a test file for OTP encryption.";
    fs::write(&input_path, input_content).unwrap();

    // 1. Generate a pad
    let mut cmd = Command::cargo_bin("otp-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("RUST_LOG", "info")
        .arg("generate")
        .arg("--pad-id")
        .arg("test_pad")
        .arg("--path")
        .arg(&pad_path)
        .arg("--size")
        .arg("1024")
        .assert()
        .success();

    // 2. Encrypt the file
    let mut cmd = Command::cargo_bin("otp-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("RUST_LOG", "info")
        .arg("encrypt")
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&encrypted_path)
        .arg("--pad-id")
        .arg("test_pad")
        .assert()
        .success()
        .stderr(predicate::str::contains("Successfully encrypted file"));

    // 3. Decrypt the file
    let mut cmd = Command::cargo_bin("otp-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("RUST_LOG", "info")
        .arg("decrypt")
        .arg("-i")
        .arg(&encrypted_path)
        .arg("-o")
        .arg(&decrypted_path)
        .arg("--metadata")
        .arg(&metadata_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Successfully decrypted file"));

    // 4. Verify the decrypted content
    let decrypted_content = fs::read_to_string(&decrypted_path).unwrap();
    assert_eq!(input_content, decrypted_content);
}