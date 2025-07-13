# Miden VM Interactive Playground

This project is an interactive web-based playground for experimenting with [Miden VM](https://github.com/0xMiden/miden-vm), built with a **Vite + React frontend** and a **Rust backend**. It allows users to write Miden assembly code, run programs, generate proofs, and visualize stack outputs directly in the browser.

---

## Features

* Write and execute Miden assembly code interactively
* Generate zero-knowledge proofs for program execution
* Visualize execution stack and program hashes
* Supports both **web deployment** and **native desktop execution** (via Tauri)
* Pre-loaded example programs for quick testing
* Fully isolated frontend/backend design for modular use

---

## Architecture Overview

| Component                 | Technology            | Purpose                                            |
| ------------------------- | --------------------- | -------------------------------------------------- |
| **Frontend**              | Vite + React          | UI playground, code editor, visualization          |
| **Backend**               | Rust + Axum (or Warp) | Miden VM execution, proof generation, API handling |
| **Native App (Optional)** | Tauri                 | For local desktop execution (optional)             |

All Miden VM interactions are handled via the backend. The frontend communicates through HTTP endpoints or Tauri commands based on the environment.

---

## Local Development Setup

### 1. Prerequisites

* **Rust** (≥ 1.78)
* **Node.js** (≥ 20) + **npm**

---

### 2. Install Frontend & Backend Dependencies

```bash
npm install
```

---

### 3. Start the frontend and backend locally(Dev-mode)

```bash
npm run dev:full
```

This starts frontend on port `3000` and backend at `3001` (make sure both ports are free)

---

### 4. Build the Frontend

```bash
npm run build:web
```

This creates a production build of the frontend inside `dist/`. The Rust backend serves this via static routes.

---

### 5. Run the Rust Backend (Local API Server)

From the project root:

```bash
cargo run --manifest-path src-tauri/Cargo.toml
```

This starts the backend server on `http://localhost:3000`, serving:

* **Frontend** at `/`
* **API endpoints** at `/api/execute`, `/api/prove`, `/api/examples`

---

### 6. Run as a Native Desktop App (Tauri)

If you want to build the native app:

```bash
npm run tauri dev
```

---

## NPM Commands

| Command             | Description                                        |
| ------------------- | -------------------------------------------------- |
| `npm install`       | Install all dependencies                           |
| `npm run dev`       | Run frontend in development mode (Vite)            |
| `npm run build:web` | Build frontend for production (outputs to `dist/`) |
| `npm run tauri dev` | Run the Tauri native app (optional)                |
| `npm run lint`      | Run code linting with `eslint`                     |
| `npm run format`    | Format frontend codebase                           |

---

## API Overview

| Endpoint        | Method | Purpose                               |
| --------------- | ------ | ------------------------------------- |
| `/api/execute`  | POST   | Run Miden program execution           |
| `/api/prove`    | POST   | Generate ZK proof for execution       |
| `/api/examples` | GET    | Retrieve predefined assembly examples |
| `/health`       | GET    | Health check endpoint                 |

---

## Example Payload (Execution)

**POST /api/execute**

```json
{
  "program": "begin push.3 push.5 add end",
  "inputs": {
    "operand_stack": []
  }
}
```

---

## Contributing

Contributions are welcome. Please follow these guidelines:

### 1. Fork & Clone the Repository

```bash
git clone https://github.com/your-org/miden-playground.git
cd miden-playground
```

### 2. Branching

Use feature branches:

```bash
git checkout -b feature/your-feature
```

---

### 3. Code Style & Linting

Ensure all code passes linting and formatting checks:

```bash
npm run lint
npm run format
```

---

### 4. Pull Request Process

* Describe the feature or fix clearly in the PR description
* Reference related issues if applicable
* Ensure tests and examples run without errors

---

## Technical Notes

* **Miden VM Execution**: Uses `Assembler`, `DefaultSourceManager`, and `execute` from `miden-vm` crates.
* **Proof Generation**: Uses Miden's `prove` functions to create zk-STARKs.
* **Error Handling**: Errors are returned via `Result<T, String>` in Rust backend and displayed in UI.
* **Static File Serving**: Rust backend serves `dist/` files via `axum::ServeDir` or `warp::fs::dir`.

---

## Deployment(Soon, currently breaks)

### Local Docker Deployment

1. Build:

```bash
docker build -t miden-playground .
```

2. Run:

```bash
docker run -p 3000:3000 miden-playground
```

This serves both frontend and backend in a single container.

---
