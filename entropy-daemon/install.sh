#!/usr/bin/env bash
set -e

echo "☢️  Geiger Entropy Oracle — Install Script"
echo "==========================================="

# Check Python
python3 --version || { echo "Python 3 required. Install with: sudo apt install python3"; exit 1; }
PYVER=$(python3 -c 'import sys; print(sys.version_info.minor)')
if [ "$PYVER" -lt 9 ]; then
  echo "Python 3.9+ required (found 3.$PYVER)"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Create venv
echo "Creating virtual environment..."
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip -q
pip install -r requirements.txt -q
echo "Dependencies installed."

# Create logs dir
mkdir -p logs

# Copy default config if not present
if [ ! -f config.toml ]; then
  echo "No config.toml found — using default. Edit entropy-daemon/config.toml before running!"
fi

# Systemd user service
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"
cat > "$SERVICE_DIR/geiger-entropy.service" << SERVICE
[Unit]
Description=Geiger Entropy Oracle Daemon
After=network.target

[Service]
Type=simple
WorkingDirectory=$SCRIPT_DIR
ExecStart=$SCRIPT_DIR/.venv/bin/python $SCRIPT_DIR/daemon.py
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
SERVICE

echo "Systemd service written to $SERVICE_DIR/geiger-entropy.service"
systemctl --user daemon-reload 2>/dev/null || true

echo ""
echo "✅  Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Edit entropy-daemon/config.toml — set watch_folder and keypair_path"
echo "  2. Test:       cd entropy-daemon && source .venv/bin/activate && python daemon.py"
echo "  3. Enable:     systemctl --user enable geiger-entropy"
echo "  4. Start:      systemctl --user start geiger-entropy"
echo "  5. Check API:  curl http://localhost:8745/health"
