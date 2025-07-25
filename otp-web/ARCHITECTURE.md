# OTP-Web Architecture (Client-Side Encryption Model)

This document outlines the revised architecture for the `otp-web` application. The new model prioritizes security by performing all cryptographic operations on the client-side (in the browser), ensuring that plaintext data is never transmitted to the server.

## 1. High-Level Overview

The application is a hybrid system. A Rust-based backend, built with Axum, is responsible for managing the state of the one-time pad (OTP) vault. It also serves the frontend, a vanilla JavaScript single-page application, by embedding the static assets directly into the binary, which allows for a self-contained deployment. The frontend handles all user interactions, file processing, and cryptographic operations.

## 2. Component Diagram

```mermaid
graph TD
    subgraph "Browser (Client-Side)"
        Frontend[HTML/JS/CSS] -- "Manages" --> UserFiles[Plaintext Files (in memory)];
        Frontend -- "Requests Pad Segment" --> WebServer;
        Frontend -- "Performs XOR" --> Crypto[In-Browser Crypto];
        UserFiles -- "Processed by" --> Crypto;
        PadSegment[Pad Segment (in memory)] -- "Used by" --> Crypto;
        Crypto -- "Outputs" --> EncryptedData[Encrypted Data];
        Frontend -- "Updates Server" --> WebServer;
    end

    subgraph "Server (Backend)"
        WebServer[Axum Web Server] -- "Serves" --> Frontend;
        WebServer -- "Provides Pad Segment" --> PadSegment;
        WebServer -- "Uses" --> CoreLogic[otp-core Library];
    end

    subgraph "Core Logic"
        CoreLogic -- "Manages" --> Vault[OTP Vault];
    end
```

## 3. Core Technologies

-   **Backend**: Axum web framework.
-   **Frontend**: Vanilla HTML, CSS, and JavaScript. The frontend is responsible for:
    -   Reading local files into memory.
    -   Requesting pad data from the server.
    -   Performing the XOR operation for encryption/decryption.
    -   Generating downloadable file blobs.
-   **Core Logic**: `otp-core` library for state management.

## 4. API Endpoints

The API is now designed to support a client-side encryption workflow.

-   `GET /api/vault/status`: Returns vault statistics.
-   `POST /api/vault/clear`: Clears the vault.
-   `GET /api/pads`: Lists all pads in the vault.
-   `POST /api/pads/generate`: Generates new one-time pads.
-   `DELETE /api/pads/:pad_id`: Deletes a specified pad from the vault.
-   `GET /api/pads/:pad_id/download`: Downloads a specific pad file for sharing.
-   `POST /api/pads/upload`: Uploads one or more pad files to the vault.
-   `POST /api/pads/request_segment`: Requests a specific segment from a pad for a client-side cryptographic operation.
-   `POST /api/pads/mark_used`: Marks a pad segment as used after a client-side operation is complete.

## 5. Data Flow

### Client-Side Encryption (File)

```mermaid
graph TD
    A[Browser: User selects file] -- "File read into memory" --> B[Browser: JS];
    B -- "POST /api/pads/request_segment (length)" --> C[Server];
    C -- "Finds segment, returns {pad_id, start, segment_data}" --> B;
    B -- "Performs XOR operation in memory" --> D[Browser: Encrypted Data];
    B -- "POST /api/pads/mark_used (pad_id, start, end)" --> C;
    D -- "Generates downloadable file" --> E[Browser: User downloads encrypted file];
    D -- "Generates downloadable metadata" --> F[Browser: User downloads metadata.json];
```

### Client-Side Encryption (Text)

```mermaid
graph TD
    A[Browser: User enters text] -- "Text encoded" --> B[Browser: JS];
    B -- "POST /api/pads/request_segment (length)" --> C[Server];
    C -- "Finds segment, returns {pad_id, start, segment_data}" --> B;
    B -- "Performs XOR in memory" --> D[Browser: Encrypted Data];
    B -- "POST /api/pads/mark_used" --> C;
    D -- "Creates JSON payload {ciphertext_base64, metadata}" --> E[Browser: Display to user];
```

### Pad Exchange Flow

```mermaid
graph TD
    subgraph Alice
        A[Browser] -- "Generates/Selects Pad" --> B[Alice's Server];
        B -- "GET /api/pads/:id/download" --> A;
        A -- "Saves Pad File" --> A;
    end
    
    A -- "Sends Pad File (Offline)" --> C[Bob's Browser];

    subgraph Bob
        C -- "POST /api/pads/upload (file)" --> D[Bob's Server];
        D -- "Adds pad to vault" --> E[otp-core Library];
        D -- "Returns Success" --> C;
    end
```

## 6. Build and Deployment

The `otp-web` application is designed to be deployed as a single, self-contained binary. This is achieved by embedding all frontend assets (HTML, CSS, JavaScript) directly into the executable at compile time.

-   **Asset Embedding**: We use the `rust-embed` crate to embed the contents of the `../static` directory directly into the application binary. This is handled by a procedural macro at the struct definition, eliminating the need for a separate `build.rs` script.
-   **Serving Embedded Files**: The Axum server uses a fallback handler to serve these embedded files from memory. This approach ensures that the application is fully self-contained, simplifying deployment and distribution across different operating systems.

## 7. Automated Releases

The `otp-web` application is built and released as a self-contained binary through an automated GitHub Actions workflow. This workflow compiles the application for both Windows and Linux, embedding all necessary static assets. The resulting binaries are published to the GitHub Releases page, allowing users to download and run the application without any additional setup.
