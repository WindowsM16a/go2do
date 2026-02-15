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

## 📥 Installation

**[Download the latest release for your platform here.](https://github.com/WindowsM16a/go2do/releases/latest)**

### 🪟 Windows

1. Download `client.exe` from the latest release.
2. Run the executable.
   _(Note: You may need to "Run anyway" if Windows Defender warns about an unrecognized app, as we are not yet code-signed)._

### 🍏 macOS

1. Download the `client` binary from the latest release.
2. Open Terminal, navigate to downloads, and make it executable:
   ```bash
   chmod +x client
   ./client
   ```
   _(Note: You may need to allow the app in System Settings > Privacy & Security if macOS blocks it)._

### 🐧 Linux

1. Download the `.deb` package.
2. Install via apt:
   ```bash
   sudo dpkg -i go2do_*.deb
   sudo apt-get install -f # Fix dependencies if needed
   ```
3. Run `go2do` from your terminal or app launcher.

### 🌐 Web & Mobile (Next.js)

- **Web App**: ✅ Live at [go2do-zeta.vercel.app](https://go2do-zeta.vercel.app).
- **Mobile**: ✅ Fully responsive web-view support for iOS and Android.

## ⌨️ Shortcuts (Desktop)

| Shortcut   | Action                                    |
| ---------- | ----------------------------------------- |
| `Ctrl + N` | Add New Task (opens window/focuses input) |
| `Esc`      | Hide Task Window                          |
| `Ctrl + Q` | Quit Application                          |
| `Ctrl + R` | Force Sync Now                            |
| `Ctrl + ,` | Open Settings (Windows/macOS fallback)    |
| `Ctrl + H` | Open Shortcuts (Windows/macOS fallback)   |

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
