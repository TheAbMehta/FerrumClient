#!/usr/bin/env bash
set -e

echo "Installing system dependencies..."
echo ""

if command -v apt-get >/dev/null; then
  echo "Detected Debian/Ubuntu"
  sudo apt-get update
  sudo apt-get install -y build-essential pkg-config libx11-dev libasound2-dev libudev-dev
elif command -v pacman >/dev/null; then
  echo "Detected Arch Linux"
  sudo pacman -S base-devel alsa-lib
elif command -v dnf >/dev/null; then
  echo "Detected Fedora"
  sudo dnf install gcc pkg-config libX11-devel alsa-lib-devel systemd-devel
else
  echo "Unsupported package manager"
  echo "Please install dependencies manually"
  exit 1
fi

echo ""
echo "Dependencies installed!"
