#!/bin/bash
set -e

echo "📦 Packaging Go2Do for local installation..."
echo "📦 Packaging Go2Do for local installation..."

# 0. Check dependencies (Debian/Ubuntu)
if [ -f /etc/debian_version ]; then
    echo "🔍 Checking system dependencies..."
    MISSING=""
    dpkg -s libgtk-4-dev &> /dev/null || MISSING="$MISSING libgtk-4-dev"
    dpkg -s libadwaita-1-dev &> /dev/null || MISSING="$MISSING libadwaita-1-dev"
    
    if [ -n "$MISSING" ]; then
        echo "⚠️ Missing dependencies: $MISSING"
        echo "   Installing them now (requires sudo)..."
        sudo apt-get update && sudo apt-get install -y $MISSING
    fi
fi
# 1. Build release binary
echo "🔨 Building release binary..."
cd client
cargo build --release
cd ..

# 2. Key Paths
INSTALL_BIN="$HOME/.local/bin"
INSTALL_ICON="$HOME/.local/share/icons/hicolor/512x512/apps"
INSTALL_DESKTOP="$HOME/.local/share/applications"
INSTALL_SHARE="$HOME/.local/share/go2do"

mkdir -p "$INSTALL_BIN"
mkdir -p "$INSTALL_ICON"
mkdir -p "$INSTALL_DESKTOP"
mkdir -p "$INSTALL_SHARE"

# 3. Install Binary
echo "🚀 Installing binary to $INSTALL_BIN..."
cp client/target/release/client "$INSTALL_BIN/go2do"

# 4. Install Resources
echo "🎨 Installing styles to $INSTALL_SHARE..."
cp client/style.css "$INSTALL_SHARE/style.css"

# 5. Install Icon (Convert JPG to PNG if ffmpeg exists)
echo "🖼️ Installing icon to $INSTALL_ICON..."
if command -v ffmpeg &> /dev/null; then
    echo "   (Using ffmpeg to convert logo_background.jpg to png)"
    ffmpeg -i logo_background.jpg -vf "scale=512:512" "$INSTALL_ICON/go2do.png" -y -loglevel error
else
    echo "   (ffmpeg not found, copying generic icon or using raw jpg)"
    # Fallback: Just copy as PNG (linux loaders often inspect headers) OR cp as jpg
    # Let's just copy the jpg as png and hope for the best, or keep it jpg if we change desktop file.
    # But desktop file says Icon=go2do which prefers png.
    cp logo_background.jpg "$INSTALL_ICON/go2do.png"
fi

# 6. Create Desktop Entry
echo "📝 Creating desktop entry..."
cat > "$INSTALL_DESKTOP/com.ideneyesa.go2do.desktop" <<EOL
[Desktop Entry]
Type=Application
Name=Go2Do
Comment=A simple to-do list app
Exec=$INSTALL_BIN/go2do
Icon=go2do
Terminal=false
Categories=Utility;GTK;
Keywords=todo;task;list;
StartupNotify=true
EOL

# 7. Update Icon Cache
echo "🔄 Updating icon cache..."
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" || true

echo "✅ Installation Complete with CSS & Icon!"
echo "You can now launch 'Go2Do' from your application menu."
