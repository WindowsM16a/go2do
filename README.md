# Go2Do

A simple, robust, and beautiful to-do application with cloud sync. Built with Rust (GTK4) and Cloudflare Workers.

## 🚀 Installation (Linux)

You can install the desktop client locally using the provided script:

```bash
# Clone the repository
git clone https://github.com/WindowsM16a/go2do.git
cd go2do

# Build & Install
./install.sh
```

This will:

- Build the optimized release binary.
- Install `go2do` to `~/.local/bin`.
- Install the icon and desktop entry so it appears in your app menu.

## 🌍 Compatibility & Distribution

### 🖥️ Desktop (Current V1)

- **Linux**: Fully supported (GNOME, KDE, XFCE, etc.).
  - **Source**: Build via `./install.sh`.
  - **Releases**: Download pre-built binaries from the [GitHub Releases](https://github.com/WindowsM16a/go2do/releases) page.
  - **Flatpak**: Manifest available (see `com.ideneyesa.go2do.yml`).
- **Windows**: Planned for V2.
- **macOS**: Planned for V2.

### 📱 Mobile & Web (Upcoming V2)

- **Web App**: A Next.js/React web version is planned for V2. This will allow access from **Android**, **iOS**, and any browser without installation.
- **Sync**: All clients (Desktop & Web) will sync to the same Cloudflare backend.

## 🛠️ Development

### Prerequisites

- Rust (Cargo)
- GTK4 development headers (`libgtk-4-dev` on Debian/Ubuntu)
- Node.js & npm (for Server)
