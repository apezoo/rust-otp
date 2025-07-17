#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[tauri::command]
fn encrypt(file_path: String, pad_id: String) -> Result<String, String> {
    // In a real app, you'd have a proper vault path
    let vault_path = "/tmp/my_test_vault";
    let output_path = format!("{}.encrypted", file_path);
    
    let result = std::process::Command::new("otp-cli")
        .arg("--vault")
        .arg(vault_path)
        .arg("encrypt")
        .arg(&file_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--pad-id")
        .arg(pad_id)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(output_path)
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn decrypt(file_path: String, metadata_path: String) -> Result<String, String> {
    let vault_path = "/tmp/my_test_vault";
    let output_path = format!("{}.decrypted", file_path);

    let result = std::process::Command::new("otp-cli")
        .arg("--vault")
        .arg(vault_path)
        .arg("decrypt")
        .arg("--input")
        .arg(&file_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--metadata")
        .arg(metadata_path)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(output_path)
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn initialize_vault() -> Result<String, String> {
    let vault_path = "/tmp/my_test_vault";
    let result = std::process::Command::new("otp-cli")
        .arg("--vault")
        .arg(vault_path)
        .arg("vault")
        .arg("init")
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok("Vault initialized successfully".to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn generate_pad() -> Result<String, String> {
    let vault_path = "/tmp/my_test_vault";
    let result = std::process::Command::new("otp-cli")
        .arg("--vault")
        .arg(vault_path)
        .arg("pad")
        .arg("generate")
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![encrypt, decrypt, initialize_vault, generate_pad])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}