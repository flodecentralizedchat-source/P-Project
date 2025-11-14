#!/bin/bash

# Install Kani model checker
echo "Installing Kani model checker..."
cargo install --locked kani-verifier
cargo kani setup

# Install Verus theorem prover (if available)
# echo "Installing Verus theorem prover..."
# cargo install --git https://github.com/verus-lang/verus.git

echo "Verification tools installation complete!"