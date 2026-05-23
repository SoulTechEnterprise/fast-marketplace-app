#!/bin/bash

set -e

REPO="SoulTechEnterprise/fast-marketplace-app"
BINARY_NAME="automatize-marketplace-linux.deb"
SERVICE_NAME="automatize-marketplace"

echo "🔍 Buscando a versão mais recente..."
LATEST=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "❌ Não foi possível obter a versão mais recente."
    exit 1
fi

echo "📦 Versão encontrada: $LATEST"
echo "⬇️  Baixando o pacote..."

curl -L "https://github.com/$REPO/releases/download/$LATEST/$BINARY_NAME" -o "/tmp/$BINARY_NAME"

echo "📁 Instalando pacote .deb..."
sudo dpkg -i "/tmp/$BINARY_NAME"

echo "⚙️  Configurando serviço systemd..."
sudo tee /etc/systemd/system/$SERVICE_NAME.service > /dev/null << EOF
[Unit]
Description=Automatize Marketplace
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/$SERVICE_NAME
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable $SERVICE_NAME
sudo systemctl restart $SERVICE_NAME

echo "✅ Instalação concluída! O serviço está rodando na porta 15137."
