const { invoke } = window.__TAURI__.shell;

document.getElementById('init-vault').addEventListener('click', () => {
    invoke('execute', { cmd: 'otp-cli', args: ['--vault', './my_vault', 'vault', 'init'] })
        .then(output => document.getElementById('output').textContent = output)
        .catch(error => document.getElementById('output').textContent = error);
});

document.getElementById('vault-status').addEventListener('click', () => {
    invoke('execute', { cmd: 'otp-cli', args: ['--vault', './my_vault', 'vault', 'status'] })
        .then(output => document.getElementById('output').textContent = output)
        .catch(error => document.getElementById('output').textContent = error);
});

document.getElementById('generate-pad').addEventListener('click', () => {
    invoke('execute', { cmd: 'otp-cli', args: ['--vault', './my_vault', 'pad', 'generate'] })
        .then(output => document.getElementById('output').textContent = output)
        .catch(error => document.getElementById('output').textContent = error);
});

document.getElementById('list-pads').addEventListener('click', () => {
    invoke('execute', { cmd: 'otp-cli', args: ['--vault', './my_vault', 'pad', 'list'] })
        .then(output => document.getElementById('output').textContent = output)
        .catch(error => document.getElementById('output').textContent = error);
});

document.getElementById('delete-pad').addEventListener('click', () => {
    const padId = document.getElementById('delete-pad-id').value;
    invoke('execute', { cmd: 'otp-cli', args: ['--vault', './my_vault', 'pad', 'delete', '--pad-id', padId] })
        .then(output => document.getElementById('output').textContent = output)
        .catch(error => document.getElementById('output').textContent = error);
});

document.getElementById('encrypt').addEventListener('click', () => {
    const input = document.getElementById('encrypt-input').value;
    const output = document.getElementById('encrypt-output').value;
    const padId = document.getElementById('encrypt-pad-id').value;
    invoke('execute', { cmd: 'otp-cli', args: ['--vault', './my_vault', 'encrypt', '--input', input, '--output', output, '--pad-id', padId] })
        .then(output => document.getElementById('output').textContent = output)
        .catch(error => document.getElementById('output').textContent = error);
});

document.getElementById('decrypt').addEventListener('click', () => {
    const input = document.getElementById('decrypt-input').value;
    const output = document.getElementById('decrypt-output').value;
    const metadata = document.getElementById('decrypt-metadata').value;
    invoke('execute', { cmd: 'otp-cli', args: ['--vault', './my_vault', 'decrypt', '--input', input, '--output', output, '--metadata', metadata] })
        .then(output => document.getElementById('output').textContent = output)
        .catch(error => document.getElementById('output').textContent = error);
});