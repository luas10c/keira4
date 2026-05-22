<div align="center">
<br>
<img alt="keira4" src="https://github.com/luas10c/keira4/blob/main/public/icon.png?raw=true" height="128">
<br><br>
<img src="https://img.shields.io/github/actions/workflow/status/luas10c/keira4/ci.yml?branch=main&style=flat-square" alt="CI">
<img src="https://badgen.net/github/tag/luas10c/keira4" alt="Release">
<img src="https://img.shields.io/badge/ESLint-3A33D1?logo=eslint" alt="eslint">
<img src="https://img.shields.io/badge/Prettier-21323b?logo=prettier&logoColor=ffffff" alt="prettier">
<img src="https://img.shields.io/github/license/luas10c/keira4" alt="github license">
</div>
<br>

Keira4 is a modern desktop tool for customizing and managing World of Warcraft private servers built with AzerothCore, TrinityCore and more.

## Quick Start

### 1. Install prerequisites

Before running the project, install the required dependencies for Tauri:

- Node.js
- npm
- Rust
- Tauri system dependencies

Follow the official Tauri prerequisites guide for your operating system:

- Windows, macOS and Linux:
  - https://v2.tauri.app/start/prerequisites/

### 2. Clone the repository:

```bash
git clone https://github.com/luas10c/keira4.git
cd keira4
```

### 3. Install dependencies:

```bash
npm install
```

### 4. Run the app in development mode:

```bash
npm run tauri -- dev
```

### 5. Build release

```bash
npm run tauri -- build
```

The generated bundles will be available in:

```bash
src-tauri/target/release/bundle
```

## Contributing

Contributions are welcome.

Please read the contribution guide before opening issues or pull requests:

- [CONTRIBUTING.md](https://github.com/luas10c/keira4/blob/main/CONTRIBUTING.md)

### Quick Start

1. Fork the repository.
2. Create a new branch:

```bash
git checkout -b feature/your-feature-name
```

3. Install dependencies:

```bash
npm install
```

4. Make your changes.
5. Run tests and checks:

```bash
npm test
npm run lint
npm run build
```

6. Commit using clear commit messages.
7. Push your branch and open a Pull Request.

## License

This project is licensed under the MIT License.
