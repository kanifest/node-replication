## Prerequisites

Install Kani and required components:


```bash
#Install Rust:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. $HOME/.cargo/env

#Install clang/cmake
sudo apt update
sudo apt install clang libclang-dev
sudo apt install cmake

# Install Kani
cargo install kani-verifier

# Install required components
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly

# Setup Kani
cargo kani setup
```

## Running Tests

To run all verification tests:

```bash
cd node-replication/node-replication
cargo kani --tests
```