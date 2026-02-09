#!/usr/bin/env bash

UDEV_RULE_FILE="/etc/udev/rules.d/99-uinput.rules"
UINPUT_GROUP="uinput"
CURRENT_USER=$(whoami)

echo 'KERNEL=="uinput", MODE="0660", GROUP="uinput"' | sudo tee "$UDEV_RULE_FILE" > /dev/null

if ! getent group "$UINPUT_GROUP" > /dev/null; then
    sudo groupadd "$UINPUT_GROUP"
fi

sudo usermod -aG "$UINPUT_GROUP" $(whoami)

sudo udevadm control --reload-rules
sudo udevadm trigger
