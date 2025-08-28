#!/bin/sh
# Build script for vvoss.dev Rust application

echo "Building vvoss.dev Rust application..."

# Build release version
cargo build --release

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Binary location: target/release/vvoss-web"
    
    # Show binary size
    ls -lh target/release/vvoss-web
else
    echo "Build failed!"
    exit 1
fi