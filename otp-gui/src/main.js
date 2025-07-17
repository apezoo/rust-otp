const { invoke } = window.__TAURI__.tauri;

async function encryptFile() {
  const filePath = await open();
  if (filePath) {
    const padId = prompt("Enter Pad ID:");
    if (padId) {
      try {
        const encryptedPath = await invoke("encrypt", { filePath, padId });
        alert(`File encrypted successfully: ${encryptedPath}`);
      } catch (error) {
        alert(`Error: ${error}`);
      }
    }
  }
}

async function decryptFile() {
  const filePath = await open();
  if (filePath) {
    const metadataPath = await open();
    if (metadataPath) {
        try {
            const decryptedPath = await invoke("decrypt", { filePath, metadataPath });
            alert(`File decrypted successfully: ${decryptedPath}`);
        } catch (error) {
            alert(`Error: ${error}`);
        }
    }
  }
}

async function initializeVault() {
  try {
    const result = await invoke("initialize_vault");
    alert(result);
  } catch (error) {
    alert(`Error: ${error}`);
  }
}

async function generatePad() {
  try {
    const padId = await invoke("generate_pad");
    alert(`Pad generated successfully: ${padId}`);
  } catch (error) {
    alert(`Error: ${error}`);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#encrypt-btn").addEventListener("click", encryptFile);
  document.querySelector("#decrypt-btn").addEventListener("click", decryptFile);
  document.querySelector("#init-vault-btn").addEventListener("click", initializeVault);
  document.querySelector("#generate-pad-btn").addEventListener("click", generatePad);
});