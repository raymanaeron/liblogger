#!/bin/bash

echo "Cleaning Rust build artifacts..."

echo "Running cargo clean..."
cargo clean

if [ $? -ne 0 ]; then
    echo "Failed to run 'cargo clean'"
    exit 1
fi

echo "Removing target directory..."
if [ -d "target" ]; then
    rm -rf target
    if [ $? -ne 0 ]; then
        echo "Failed to remove target directory"
        exit 1
    fi
    echo "Target directory removed successfully."
else
    echo "Target directory not found. Nothing to remove."
fi

echo "Cleanup completed successfully."
