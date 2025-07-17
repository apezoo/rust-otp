#![allow(missing_docs)]
use std::fs;
use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_generate_multiple_pads() {
    // 1. Setup
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("my_test_vault");
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("init")
        .assert().success();

    // 2. Generate 3 pads at once
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("generate")
        .arg("--count").arg("3")
        .assert().success();

    // 3. Verify that 3 pads were created
        Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("status")
        .assert().success()
        .stdout(predicate::str::contains("Total Pads: 3"));
}

#[test]
fn test_automatic_pad_selection_for_encryption() {
    // 1. Setup
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("my_test_vault");
    let input_content = "This test relies on automatic pad selection.";
    let input_path = temp_dir.path().join("auto_select_input.txt");
    fs::write(&input_path, input_content).expect("Failed to write to input path");

    // 2. Initialize vault and generate a pad
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("init")
        .assert().success();
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("generate")
        .assert().success();

    // 3. Encrypt without specifying a pad ID and capture stdout
    let encrypted_path = temp_dir.path().join("auto_select_encrypted.bin");
    let encrypt_output = Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("encrypt")
        .arg(&input_path)
        .arg("--output").arg(&encrypted_path)
        .output().expect("Failed to encrypt file");

    assert!(encrypt_output.status.success());
 
    let _stdout = String::from_utf8(encrypt_output.stdout).expect("Failed to read stdout");
    let metadata_path = temp_dir.path().join("auto_select_encrypted.bin.metadata.json");
    
    // 4. Decrypt and verify
    let decrypted_path = temp_dir.path().join("auto_select_decrypted.txt");
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("decrypt")
        .arg("--input").arg(&encrypted_path)
        .arg("--output").arg(&decrypted_path)
        .arg("--metadata").arg(&metadata_path)
        .assert().success();
    
    let decrypted_content = fs::read_to_string(&decrypted_path).expect("Failed to read decrypted file");
    assert_eq!(input_content, decrypted_content);
}

#[test]
fn test_pad_is_moved_when_fully_consumed() {
    // 1. Setup - create a tiny pad (1KB)
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("my_test_vault");
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("init")
        .assert().success();
    
    // For this test, we need a known pad size, so we can't use the default MB.
    // We'll generate a pad and manually create a small file.
    let generate_output = Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("generate").arg("--size").arg("1") // 1MB is smallest size
        .output().expect("Failed to generate pad");
    let pad_id = String::from_utf8(generate_output.stdout).expect("Failed to read pad id from stdout").trim().to_string();

    // Create a file that will use up a large portion of the pad
    let large_chunk_size = 1024 * 512; // 0.5 MB
    let large_input_content = vec![65u8; large_chunk_size];
    let large_input_path = temp_dir.path().join("large_input.txt");
    fs::write(&large_input_path, &large_input_content).expect("Failed to write large input file");
    
    // 2. Encrypt the large file
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("encrypt").arg(&large_input_path)
        .arg("--pad-id").arg(&pad_id)
        .assert().success();
    
    // The pad should still be available
    let available_pad_path = vault_path.join("pads/available").join(format!("{pad_id}.pad"));
    assert!(available_pad_path.exists());

    // 3. Create another large file that will consume the rest of the pad
    let second_large_chunk_size = 1024 * 512; // 0.5MB
    let second_large_input_content = vec![66u8; second_large_chunk_size];
    let second_large_input_path = temp_dir.path().join("second_large_input.txt");
    fs::write(&second_large_input_path, &second_large_input_content).expect("Failed to write second large input file");
    
    // 4. Encrypt the second file
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("encrypt").arg(&second_large_input_path)
        .arg("--pad-id").arg(&pad_id)
        .assert().success()
        .stdout(predicate::str::contains("fully consumed"));

    // 5. Verify the pad file has been moved
    let used_pad_path = vault_path.join("pads/used").join(format!("{pad_id}.pad"));
    assert!(!available_pad_path.exists(), "Pad should no longer be in available dir");
    assert!(used_pad_path.exists(), "Pad should now be in used dir");

    // 6. Verify vault status shows 1 used pad
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("status")
        .assert().success()
        .stdout(predicate::str::contains("Fully Used: 1"));
}

#[test]
fn test_manual_decryption_without_metadata() {
    // 1. Setup
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let vault_path = temp_dir.path().join("my_test_vault");
    let input_content = "This is a manual decryption test.";
    let input_path = temp_dir.path().join("manual_input.txt");
    fs::write(&input_path, input_content).expect("Failed to write input file");

    // 2. Initialize vault and generate a pad
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("vault").arg("init")
        .assert().success();
    let generate_output = Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("pad").arg("generate")
        .output().expect("Failed to generate pad");
    let pad_id = String::from_utf8(generate_output.stdout).expect("Failed to read pad id from stdout").trim().to_string();

    // 3. Encrypt the file
    let encrypted_path = temp_dir.path().join("manual_encrypted.bin");
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("encrypt").arg(&input_path)
        .arg("--output").arg(&encrypted_path)
        .arg("--pad-id").arg(&pad_id)
        .assert().success();

    // 4. Decrypt without metadata
    let decrypted_path = temp_dir.path().join("manual_decrypted.txt");
    Command::cargo_bin("otp-cli").expect("Failed to find otp-cli binary")
        .arg("--vault").arg(&vault_path)
        .arg("decrypt")
        .arg("--input").arg(&encrypted_path)
        .arg("--output").arg(&decrypted_path)
        .arg("--pad-id").arg(&pad_id)
        .arg("--length").arg(input_content.len().to_string())
        .assert().success();

    // 5. Verify
    let decrypted_content = fs::read_to_string(&decrypted_path).expect("Failed to read decrypted file");
    assert_eq!(input_content, decrypted_content);
}