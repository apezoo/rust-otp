// --- Helpers ---
const base64ToArr = (b64) => new Uint8Array(atob(b64).split('').map(c => c.charCodeAt(0)));
const arrToBase64 = (arr) => btoa(String.fromCharCode.apply(null, arr));
const downloadBlob = (blob, filename) => {
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
};
const notify = (message, type = 'info') => {
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.textContent = message;
    document.body.appendChild(notification);
    setTimeout(() => notification.remove(), 3000);
};

// --- API Functions ---
async function getVaultStatus() {
    try {
        const response = await fetch('/api/vault/status');
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        const status = await response.json();
        const statusDiv = document.getElementById('vault-status');
        statusDiv.innerHTML = `
            <h3>Vault Status</h3>
            <p><strong>Total Pads:</strong> ${status.total_pads} | <strong>Available:</strong> ${status.available_pads}</p>
            <p><strong>Total Storage:</strong> ${(status.total_storage_bytes / 1024 / 1024).toFixed(2)} MB</p>
        `;
    } catch (error) {
        notify('Error loading vault status.', 'error');
    }
}

async function listPads() {
    try {
        const response = await fetch('/api/pads');
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        const pads = await response.json();
        const padsDiv = document.getElementById('pads-list');
        if (pads.length === 0) {
            padsDiv.innerHTML = '<p>No pads found in vault.</p>';
            return;
        }

        let table = '<h3>Available Pads</h3><table><tr><th>ID</th><th>Size (MB)</th><th>Usage</th><th>Actions</th></tr>';
        for (const pad of pads) {
            const totalUsed = pad.used_segments.reduce((acc, s) => acc + (s.end - s.start), 0);
            const usedPercent = pad.size > 0 ? (totalUsed / pad.size * 100).toFixed(2) : 0;
            table += `
                <tr data-pad-id="${pad.id}">
                    <td>${pad.id}</td>
                    <td>${(pad.size / 1024 / 1024).toFixed(2)}</td>
                    <td>${usedPercent}%</td>
                    <td>
                        <button class="download-pad-btn">Download</button>
                        <button class="delete-pad-btn">Delete</button>
                    </td>
                </tr>
            `;
        }
        table += '</table>';
        padsDiv.innerHTML = table;

        document.querySelectorAll('.delete-pad-btn').forEach(b => b.addEventListener('click', () => deletePad(b.closest('tr').dataset.padId)));
        document.querySelectorAll('.download-pad-btn').forEach(b => b.addEventListener('click', () => downloadPad(b.closest('tr').dataset.padId)));
    } catch (error) {
       notify('Error loading pads.', 'error');
    }
}

// --- Pad Management ---
async function generatePad() {
    const size = parseInt(prompt("Enter pad size in MB:", "1"));
    const count = parseInt(prompt("Enter number of pads to generate:", "1"));
    if (isNaN(size) || isNaN(count)) return notify("Invalid input.", 'error');

    try {
        const response = await fetch('/api/pads/generate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ size, count }),
        });
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        const result = await response.json();
        notify(`Successfully generated pads: \n${result.pad_ids.join('\n')}`);
        getVaultStatus();
        listPads();
    } catch (error) {
        notify('Error generating pads.', 'error');
    }
}

async function deletePad(padId) {
    if (!confirm(`Are you sure you want to delete pad ${padId}?`)) return;
    try {
        const response = await fetch(`/api/pads/${padId}`, { method: 'DELETE' });
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        notify("Pad deleted successfully.");
        getVaultStatus();
        listPads();
    } catch (error) {
        notify('Error deleting pad.', 'error');
    }
}

async function downloadPad(padId) {
    try {
        const response = await fetch(`/api/pads/${padId}/download`);
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        const blob = await response.blob();
        downloadBlob(blob, `${padId}.pad`);
    } catch (error) {
        notify('Error downloading pad.', 'error');
    }
}

async function uploadPads() {
    const files = document.getElementById('upload-pads-input').files;
    if (files.length === 0) return notify("Please select files to upload.", 'error');

    const formData = new FormData();
    for (const file of files) formData.append('pads', file);

    try {
        const response = await fetch('/api/pads/upload', { method: 'POST', body: formData });
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || `HTTP error! status: ${response.status}`);
        }
        const result = await response.json();
        notify(`Successfully imported pads: \n${result.imported_pads.join('\n')}`);
        getVaultStatus();
        listPads();
    } catch (error) {
        notify(`Error uploading pads: ${error.message}`, 'error');
    }
}

async function clearVault() {
    if (!confirm("Are you sure you want to delete all pads and clear the vault? This cannot be undone.")) return;
    try {
        const response = await fetch('/api/vault/clear', { method: 'POST' });
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        notify("Vault cleared successfully.");
        getVaultStatus();
        listPads();
    } catch (error) {
        notify('Error clearing vault.', 'error');
    }
}

// --- Client-Side Crypto ---
async function encrypt(plaintext, padId = null) {
    const length = plaintext.length;

    const segmentResponse = await fetch('/api/pads/request_segment', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ length, pad_id: padId }),
    });
    if (!segmentResponse.ok) {
        const err = await segmentResponse.json();
        throw new Error(err.error || "Could not get pad segment.");
    }
    const { pad_id, start, segment_data } = await segmentResponse.json();
    const padSegment = new Uint8Array(segment_data);

    const ciphertext = new Uint8Array(length);
    for (let i = 0; i < length; i++) {
        ciphertext[i] = plaintext[i] ^ padSegment[i];
    }
    
    await fetch('/api/pads/mark_used', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ pad_id, start, end: start + length }),
    });

    getVaultStatus();
    listPads();

    return { ciphertext, metadata: { pad_id, start, length } };
}

async function decrypt(ciphertext, metadata) {
    const { pad_id, start, length } = metadata;

    const segmentResponse = await fetch('/api/pads/request_segment', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ pad_id, length, start }),
    });
    if (!segmentResponse.ok) {
        const err = await segmentResponse.json();
        throw new Error(err.error || "Could not get pad segment.");
    }
    const { segment_data } = await segmentResponse.json();
    const padSegment = new Uint8Array(segment_data);

    const plaintext = new Uint8Array(length);
    for (let i = 0; i < length; i++) {
        plaintext[i] = ciphertext[i] ^ padSegment[i];
    }

    return plaintext;
}

// --- UI Event Handlers ---
async function handleEncryptText() {
    const text = document.getElementById('encrypt-text-input').value;
    if (!text) return notify("Please enter text to encrypt.", 'error');
    const plaintext = new TextEncoder().encode(text);

    try {
        const { ciphertext, metadata } = await encrypt(plaintext);
        const payload = { ciphertext_base64: arrToBase64(ciphertext), metadata };
        document.getElementById('decrypt-text-input').value = JSON.stringify(payload, null, 2);
        notify("Text encrypted successfully.");
    } catch (error) {
        notify(`Encryption failed: ${error.message}`, 'error');
    }
}

async function handleDecryptText() {
    const text = document.getElementById('decrypt-text-input').value;
    if (!text) return notify("Please enter JSON payload.", 'error');

    try {
        const { ciphertext_base64, metadata } = JSON.parse(text);
        const ciphertext = base64ToArr(ciphertext_base64);
        const plaintext = await decrypt(ciphertext, metadata);
        document.getElementById('encrypt-text-input').value = new TextDecoder().decode(plaintext);
        notify("Text decrypted successfully.");
    } catch (error) {
        notify(`Decryption failed: ${error.message}`, 'error');
    }
}

async function handleEncryptFile() {
    const file = document.getElementById('encrypt-file-input').files[0];
    if (!file) return notify("Please select a file to encrypt.", 'error');

    const plaintext = new Uint8Array(await file.arrayBuffer());
    try {
        const { ciphertext, metadata } = await encrypt(plaintext);
        const encryptedFileBlob = new Blob([ciphertext], { type: 'application/octet-stream' });
        const metadataBlob = new Blob([JSON.stringify(metadata, null, 2)], { type: 'application/json' });
        downloadBlob(encryptedFileBlob, `${file.name}.enc`);
        downloadBlob(metadataBlob, `${file.name}.enc.metadata.json`);
        notify("File encrypted successfully.");
    } catch (error) {
        notify(`File encryption failed: ${error.message}`, 'error');
    }
}

async function handleDecryptFile() {
    const encryptedFile = document.getElementById('decrypt-file-input').files[0];
    const metadataFile = document.getElementById('decrypt-metadata-file-input').files[0];

    if (!encryptedFile) return notify("Please select an encrypted file.", 'error');

    try {
        const ciphertext = new Uint8Array(await encryptedFile.arrayBuffer());
        let metadata;

        if (metadataFile) {
            metadata = JSON.parse(await metadataFile.text());
        } else {
            const pad_id = prompt("Please enter the Pad ID:");
            const start = parseInt(prompt("Please enter the start offset:"));
            const length = ciphertext.byteLength;
            if (!pad_id || isNaN(start)) return notify("Missing information for manual decryption.", 'error');
            metadata = { pad_id, start, length };
        }
        
        const plaintext = await decrypt(ciphertext, metadata);
        const decryptedFileBlob = new Blob([plaintext], { type: 'application/octet-stream' });
        const originalName = encryptedFile.name.endsWith('.enc') ? encryptedFile.name.slice(0, -4) : `${encryptedFile.name}.dec`;
        downloadBlob(decryptedFileBlob, originalName);
        notify("File decrypted successfully.");
    } catch (error) {
        notify(`File decryption failed: ${error.message}`, 'error');
    }
}

window.addEventListener("DOMContentLoaded", () => {
    getVaultStatus();
    listPads();
    document.getElementById('encrypt-text-input').value = '';
    document.getElementById('decrypt-text-input').value = '';
    document.getElementById('encrypt-file-input').value = '';
    document.getElementById('decrypt-file-input').value = '';
    document.getElementById('decrypt-metadata-file-input').value = '';
    document.getElementById('upload-pads-input').value = '';

    document.querySelector("#generate-pad-btn").addEventListener("click", generatePad);
    document.querySelector("#list-pads-btn").addEventListener("click", listPads);
    document.querySelector("#upload-pads-btn").addEventListener("click", uploadPads);
    document.querySelector("#clear-vault-btn").addEventListener("click", clearVault);
    document.querySelector("#encrypt-text-btn").addEventListener("click", handleEncryptText);
    document.querySelector("#decrypt-text-btn").addEventListener("click", handleDecryptText);
    document.querySelector("#encrypt-file-btn").addEventListener("click", handleEncryptFile);
    document.querySelector("#decrypt-file-btn").addEventListener("click", handleDecryptFile);
});