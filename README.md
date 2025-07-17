# One-Time Pad (OTP) Encryption Tool

This repository contains a suite of tools for one-time pad (OTP) encryption, including a command-line interface (CLI) and a web-based user interface.

## Project Structure

-   `otp-core`: A shared library containing all the core logic for pad generation, state management, and cryptographic operations.
-   `otp-cli`: A command-line interface for interacting with the OTP vault.
-   `otp-web`: A web server that provides a user-friendly, client-side encryption interface.
-   `static`: Contains the HTML, CSS, and JavaScript for the web interface.

## Getting Started

### Web Application

To run the web application:

```bash
cargo run -p otp-web
```

### Command-Line Interface

To use the CLI:

```bash
cargo run -p otp-cli -- --help