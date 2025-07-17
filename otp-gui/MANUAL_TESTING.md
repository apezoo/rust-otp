# Manual GUI Testing Plan

This document provides a step-by-step guide for manually testing the `otp-gui` application to ensure its core functionality is working correctly.

## Prerequisites

1.  You have successfully installed all the required system dependencies as outlined in `otp-gui/README.md`.
2.  You have built the `otp-cli` executable.
3.  You have built and launched the `otp-gui` application using `cargo tauri dev`.

## Test Workflow

This workflow simulates a complete user session, from creating a vault to encrypting and decrypting a file.

### 1. Initialize the Vault

1.  Launch the `otp-gui` application.
2.  In the "Vault" section, click the **Initialize Vault** button.
3.  Verify that a `my_vault` directory is created in the `otp-cli` directory.
4.  Check the "Output" section in the GUI for a success message.

### 2. Generate a Pad

1.  In the "Pads" section, click the **Generate Pad** button.
2.  Verify that a new `.pad` file is created in the `my_vault/pads/available/` directory.
3.  The "Output" section should display the ID of the newly generated pad. Copy this ID for the next steps.

### 3. List Pads

1.  In the "Pads" section, click the **List Pads** button.
2.  Verify that the "Output" section displays a table containing the pad you just created.

### 4. Create a Test File

1.  In the `otp-gui` directory, create a new file named `test_input.txt`.
2.  Add the following text to the file: `This is a manual test of the OTP GUI.`

### 5. Encrypt the File

1.  In the "Encryption" section of the GUI:
    *   **Input File**: Enter `../test_input.txt`
    *   **Output File**: Enter `../encrypted_message.bin`
    *   **Pad ID**: Paste the pad ID you copied in step 2.
2.  Click the **Encrypt** button.
3.  Verify that `encrypted_message.bin` and `encrypted_message.bin.metadata.json` are created in the `otp-cli` directory.
4.  Check the "Output" section for a success message.

### 6. Check Vault Status

1.  In the "Vault" section, click the **Vault Status** button.
2.  Verify that the "Output" section shows that a portion of the pad has been used (e.g., "Used: 0.00 MB" should now show a non-zero value).

### 7. Decrypt the File

1.  In the "Decryption" section of the GUI:
    *   **Input File**: Enter `../encrypted_message.bin`
    *   **Output File**: Enter `../decrypted_message.txt`
    *   **Metadata File**: Enter `../encrypted_message.bin.metadata.json`
2.  Click the **Decrypt** button.
3.  Verify that a `decrypted_message.txt` file is created in the `otp-cli` directory.
4.  Open the `decrypted_message.txt` file and confirm its content is identical to the original test file: `This is a manual test of the OTP GUI.`
5.  Check the "Output" section for a success message.

### 8. Delete the Pad

1.  In the "Pads" section:
    *   **Pad ID to Delete**: Paste the pad ID from step 2.
2.  Click the **Delete Pad** button.
3.  Check the "Output" section for a success message.

### 9. Confirm Deletion

1.  In the "Pads" section, click the **List Pads** button again.
2.  Verify that the "Output" section now displays a "No pads found" message.

If all of these steps are completed successfully, the GUI is working as expected.