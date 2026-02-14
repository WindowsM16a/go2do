# Go2Do V2

A simple, robust, and beautiful to-do application with cloud sync. Built with Rust (GTK4), Next.js, and Cloudflare Workers.

## 🚀 Installation (Linux)

You can install the desktop client locally using the provided script:

```bash
# Clone the repository
git clone https://github.com/WindowsM16a/go2do.git
cd go2do

# Build & Install
./install.sh
```

### Debian/Ubuntu Package (.deb)

If you prefer a system-wide installation, you can install the `.deb` package:

```bash
sudo dpkg -i dist/go2do_0.1.0-1_amd64.deb
```

## 🌍 Platform Status

### 🖥️ Desktop (Rust + GTK4)

- **Linux**: ✅ Native support. Package via `cargo-deb`.
- **Windows**: 🛠️ Planned. Requires CI/CD pipeline (GitHub Actions) for stable GTK4 bundling.
- **macOS**: 🛠️ Planned. Requires CI/CD pipeline (GitHub Actions) for signing and notarization.

### 🌐 Web & Mobile (Next.js)

- **Web App**: ✅ Live at `https://go2do.app` (or your deployment).
- **Mobile**: ✅ Fully responsive web-view support for iOS and Android.

## ⌨️ Shortcuts (Desktop)

| Shortcut   | Action                                    |
| ---------- | ----------------------------------------- |
| `Ctrl + N` | Add New Task (opens window/focuses input) |
| `Esc`      | Hide Task Window                          |
| `Ctrl + Q` | Quit Application                          |
| `Ctrl + R` | Force Sync Now                            |

## 🛠️ Development

### Prerequisites

- **Desktop**: Rust, GTK4 dev headers (`libgtk-4-dev`), `libadwaita-1-dev`.
- **Web**: Node.js 18+, npm.
- **Server**: Cloudflare Wrangler CLI.

### Building

```bash
# Desktop
cd client && cargo build --release

# Web
cd web && npm install && npm run dev

# Server
cd server && npm run dev
```
