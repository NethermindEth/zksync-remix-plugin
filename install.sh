sudo apt-get update
sudo  apt-get install -y curl
sudo  apt-get install -y git
sudo  apt-get install -y software-properties-common
sudo  apt-get install -y libgmp-dev
sudo apt-get install screen -y

# Install Rust
sudo  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Install nodejs
sudo  curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo  apt-get install -y nodejs

# Install pnpm
sudo  npm install -g pnpm

deactivate