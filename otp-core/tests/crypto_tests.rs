#![allow(missing_docs)]
use otp_core::crypto;

#[test]
fn test_encryption_decryption_roundtrip() {
    let plaintext = b"Hello, world!";
    let pad = (0..plaintext.len()).map(|i| ((i * 7) % 256) as u8).collect::<Vec<u8>>();

    let ciphertext = crypto::xor(plaintext, &pad);
    let decrypted_plaintext = crypto::xor(&ciphertext, &pad);

    assert_eq!(plaintext, &decrypted_plaintext[..]);
}

#[test]
fn test_file_encryption_decryption_simulation() {
    // 1. Simulate frontend reading a file
    let original_content = b"This is a test file for encryption.";
    let plaintext = original_content.to_vec();
    let length = plaintext.len();

    // 2. Simulate requesting and receiving a pad segment
    let pad_id = "test-pad-id".to_string();
    let start = 123;
    let pad_segment = (0..length).map(|i| ((i * 3) % 256) as u8).collect::<Vec<u8>>();

    // 3. Simulate client-side encryption
    let ciphertext = crypto::xor(&plaintext, &pad_segment);
    let metadata = format!(r#"{{"pad_id":"{pad_id}","start":{start},"length":{length}}}"#);

    // 4. Simulate client-side decryption
    let received_ciphertext = ciphertext;
    let received_metadata: serde_json::Value = serde_json::from_str(&metadata).unwrap();
    
    let received_pad_id = received_metadata["pad_id"].as_str().unwrap();
    let received_start = received_metadata["start"].as_u64().unwrap() as usize;
    let received_length = received_metadata["length"].as_u64().unwrap() as usize;

    assert_eq!(pad_id, received_pad_id);
    assert_eq!(start, received_start);
    assert_eq!(length, received_length);

    // 5. Decrypt and verify
    let decrypted_plaintext = crypto::xor(&received_ciphertext, &pad_segment);
    assert_eq!(original_content, &decrypted_plaintext[..]);
}