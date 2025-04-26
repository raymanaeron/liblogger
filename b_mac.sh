#!/bin/bash

BUILD_TYPE="debug"

if [ "$1" == "--release" ]; then
    BUILD_TYPE="release"
    echo "Building in RELEASE mode"
else
    echo "Building in DEBUG mode"
fi

echo "Building Rusty Logger..."
cargo build $1

if [ $? -ne 0 ]; then
    echo "Build failed"
    exit 1
fi

echo "Copying config file to target/$BUILD_TYPE directory..."
cp app_config.toml target/$BUILD_TYPE/

if [ $? -ne 0 ]; then
    echo "Failed to copy app_config.toml to target/$BUILD_TYPE/"
    exit 1
fi

echo "Build completed successfully."
echo "Configuration file copied to target/$BUILD_TYPE/app_config.toml"
