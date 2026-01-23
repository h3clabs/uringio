#!/bin/bash

# Update
apt-get update

# Sudo
apt-get install -y sudo
sed -i 's|%sudo.ALL=(ALL:ALL).ALL|%sudo ALL=(ALL:ALL) NOPASSWD: ALL|' /etc/sudoers
# useradd -rmU ${DK_USER} -u 1000 -G sudo

# Timezone & Locale
sudo apt-get install -y locales tzdata
sudo localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8

# Common tools
sudo apt-get install -y ca-certificates curl vim git git-lfs
sudo apt-get install -y build-essential cmake libssl-dev pkg-config

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y

# Build
sudo apt-get install -y libclang-dev
