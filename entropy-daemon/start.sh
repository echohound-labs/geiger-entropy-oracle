#!/bin/bash
# Geiger Entropy Oracle — Mainnet Startup Script
# Echo Hound Labs ☢️

echo "☢️  Starting Geiger Entropy Oracle v3..."

# Check Geiger counter is connected
if [ ! -e /dev/ttyUSB0 ]; then
    echo "❌ GMC-500 not detected at /dev/ttyUSB0"
    echo "   Connect your Geiger counter via USB and try again"
    exit 1
fi

# Check config exists
if [ ! -f ./config-mainnet.toml ]; then
    echo "❌ config-mainnet.toml not found"
    echo "   Copy config.toml to config-mainnet.toml and configure it"
    exit 1
fi

# Kill any existing daemon
pkill -f daemon.py 2>/dev/null
sleep 1

# Start daemon
CONFIG_PATH=./config-mainnet.toml \
SUBMIT_SCRIPT=./submit_entropy_mainnet.js \
python3 daemon.py > logs/mainnet-daemon.log 2>&1 &

sleep 5

# Check health
HEALTH=$(curl -s http://localhost:8746/health)
if [ $? -eq 0 ]; then
    echo "✓ Oracle is running!"
    echo $HEALTH
else
    echo "❌ Oracle failed to start"
    echo "Check logs: tail -f logs/mainnet-daemon.log"
fi
