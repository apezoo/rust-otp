# OTP-GUI

This is a graphical user interface for the `otp-cli` application, built with [Tauri](https://tauri.app/).

## Prerequisites

Before you can build and run this application, you will need to install the following system dependencies:

### Debian/Ubuntu

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf
```

### Fedora/CentOS/RHEL

```bash
sudo dnf install -y \
    curl \
    wget \
    openssl-devel \
    gtk3-devel \
    webkit2gtk4.1-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    patchelf
```

## Building and Running the Application

1.  **Navigate to the `otp-gui` directory:**
    ```bash
    cd otp-gui
    ```

2.  **Install the Rust dependencies:**
    ```bash
    cargo fetch
    ```

3.  **Build and run the application in development mode:**
    ```bash
    cargo tauri dev
    ```

4.  **Build the application for production:**
    ```bash
    cargo tauri build
    ```

The production-ready application will be located in the `otp-gui/src-tauri/target/release/bundle/` directory.