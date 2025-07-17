# One-Time Pad (OTP) Encryption Tool

This repository provides a robust and secure suite of tools for one-time pad (OTP) encryption, designed for both command-line and web-based environments. Our mission is to offer a powerful, user-friendly, and highly secure solution for protecting sensitive data.

## 🚀 Features

-   **Cross-Platform**: A shared `otp-core` library ensures consistent and reliable cryptographic operations across all our tools.
-   **Command-Line Interface (CLI)**: A powerful CLI (`otp-cli`) for programmatic access to all encryption and decryption functionalities.
-   **Web-Based Interface**: A user-friendly web interface (`otp-web`) for easy, client-side encryption and decryption.
-   **Secure Pad Generation**: Generate cryptographically secure one-time pads of any size.
-   **State Management**: The system intelligently manages the state of the one-time pads, ensuring that no part of a pad is ever reused.
-   **Streaming Support**: Efficiently encrypt and decrypt large files using a streaming approach, minimizing memory usage.

## 🛡️ Security

The one-time pad is a theoretically unbreakable encryption algorithm when used correctly. This implementation is designed to enforce the correct usage of one-time pads by:

-   **Preventing Pad Reuse**: The system is architected to make it impossible to reuse any portion of a one-time pad.
-   **Cryptographically Secure Randomness**: One-time pads are generated using a cryptographically secure pseudo-random number generator (CSPRNG).
-   **No Key Distribution**: This tool does not handle the distribution of one-time pads. It is the user's responsibility to securely share the one-time pad with the intended recipient.

## 🤝 Contributing

We welcome contributions from the community. If you would like to contribute to the project, please follow these steps:

1.  Fork the repository.
2.  Create a new branch for your feature or bug fix.
3.  Make your changes and ensure that all tests pass.
4.  Submit a pull request with a clear description of your changes.

## 📜 License

This project is licensed under the terms of the MIT license. See the [LICENSE](LICENSE) file for more details.

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

## 📦 Building Binaries

To build the binaries for both Linux and Windows, run the following command:

```bash
./build.sh
```

This will create `otp-cli` and `otp-web` binaries for both platforms in the `precompiled_binaries` directory. The `otp-web` binary for Windows will have the static assets embedded in it, so it can be run without the `static` directory.