# One-Time Pad (OTP) Encryption Tool

This repository provides a robust and secure suite of tools for one-time pad (OTP) encryption, designed for both command-line and web-based environments. Our mission is to offer a powerful, user-friendly, and highly secure solution for protecting sensitive data.

## 🚀 Features

-   **Cross-Platform**: A shared `otp-core` library ensures consistent and reliable cryptographic operations across all our tools.
-   **Command-Line Interface (CLI)**: A powerful CLI (`otp-cli`) for programmatic access to all encryption and decryption functionalities.
-   **Web-Based Interface**: A user-friendly web interface (`otp-web`) for easy, client-side encryption and decryption.
-   **Secure Pad Generation**: Generate cryptographically secure one-time pads of any size.
-   **State Management**: The system intelligently manages the state of the one-time pads, ensuring that no part of a pad is ever reused.
-   **Streaming Support**: Efficiently encrypt and decrypt large files using a streaming approach, minimizing memory usage.

## Releases

Pre-compiled binaries for our v1.0.0 release are provided below. For future updates, please check the [GitHub Releases page](https://github.com/apezoo/rust-otp/releases).

For SHA256 checksums, please see the `sha256sums.txt` file attached to the latest release on our [GitHub Releases page](https://github.com/apezoo/rust-otp/releases/latest).

## 🛡️ Security

The one-time pad is a theoretically unbreakable encryption algorithm when used correctly. This implementation is designed to enforce the correct usage of one-time pads by:

-   **Preventing Pad Reuse**: The system is architected to make it impossible to reuse any portion of a one-time pad.
-   **Cryptographically Secure Randomness**: One-time pads are generated using a cryptographically secure pseudo-random number generator (CSPRNG).
-   **No Key Distribution**: This tool does not handle the distribution of one-time pads. It is the user's responsibility to securely share the one-time pad with the intended recipient.

## Choosing the Right Binary

*   `otp-cli`: The command-line interface for Linux.
*   `otp-cli.exe`: The command-line interface for Windows.
*   `otp-web`: The web-based interface for Linux.
*   `otp-web.exe`: The web-based interface for Windows.

## 🤝 Contributing

We welcome contributions from the community. If you would like to contribute to the project, please follow these steps:

1.  Fork the repository.
2.  Create a new branch for your feature or bug fix.
3.  Make your changes and ensure that all tests pass.
4.  Submit a pull request with a clear description of your changes.

**Note on Build Artifacts**: This project uses a `.gitignore` file to exclude build artifacts (the `target` directory) from the repository. Please ensure you do not force-add these files to your commits.

## 📜 License

This project is licensed under the terms of the `GNU Affero General Public License v3.0`. See the [LICENSE](LICENSE.md) file for more details.

## 📚 Project Documentation

For more detailed information about the architecture and usage of each component, please refer to the following documents:

- **Core Library (`otp-core`)**
  - [**README**](otp-core/README.md): Core concepts and usage.
  - [**ARCHITECTURE**](otp-core/ARCHITECTURE.md): In-depth technical design of the core components.

- **Command-Line Interface (`otp-cli`)**
  - [**ARCHITECTURE**](otp-core/ARCHITECTURE.md): Design and command structure of the CLI tool. Note: The `otp-cli` architecture is documented in the `otp-core` directory.

- **Web Application (`otp-web`)**
  - [**README**](otp-web/README.md): Setup and usage for the web interface.
  - [**ARCHITECTURE**](otp-web/ARCHITECTURE.md): Architecture of the web application, including API endpoints and data flow.

## 🏁 Getting Started

### Web Application

To run the web application, execute the following command:

```bash
cargo run -p otp-web
```

### Command-Line Interface

To use the CLI, you can get a list of all available commands by running:

```bash
cargo run -p otp-cli -- --help
```

## 📦 Building from Source

If you prefer to build the binaries from source, you can use the provided script:

```bash
./build.sh
```

This will create `otp-cli` and `otp-web` binaries for both platforms in the `precompiled_binaries` directory. The `otp-web` binaries have the static assets embedded, so they can be run without the `static` directory.