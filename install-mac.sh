#!/bin/bash

set -e

REPO="SoulTechEnterprise/fast-marketplace-app"
BINARY_NAME="automatize-marketplace-macos"
INSTALL_DIR="/usr/local/bin"
SERVICE_NAME="automatize-marketplace"
PLIST_PATH="$HOME/Library/LaunchAgents/$SERVICE_NAME.plist"

echo "🔍 Buscando a versão mais recente..."
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "❌ Não foi possível obter a versão mais recente."
    exit 1
fi

echo "📦 Versão encontrada: $LATEST"
echo "⬇️  Baixando o binário..."

curl -L "https://github.com/$REPO/releases/download/$LATEST/$BINARY_NAME" -o "/tmp/$SERVICE_NAME"
chmod +x "/tmp/$SERVICE_NAME"

echo "📁 Instalando em $INSTALL_DIR..."
sudo mv "/tmp/$SERVICE_NAME" "$INSTALL_DIR/$SERVICE_NAME"

echo "⚙️  Configurando para iniciar automaticamente..."
cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$SERVICE_NAME</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/$SERVICE_NAME</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$HOME/Library/Logs/$SERVICE_NAME.log</string>
    <key>StandardErrorPath</key>
    <string>$HOME/Library/Logs/$SERVICE_NAME.log</string>
</dict>
</plist>
EOF

launchctl unload "$PLIST_PATH" 2>/dev/null || true
launchctl load "$PLIST_PATH"

echo "✅ Instalação concluída! O serviço está rodando na porta 15137."
