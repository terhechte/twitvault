#!/bin/sh

set -e

# Compile for older macOS Versions
# See: https://users.rust-lang.org/t/compile-rust-binary-for-older-versions-of-mac-osx/38695/2
export MACOSX_DEPLOYMENT_TARGET=10.10

rm -rf target/release/bundle/osx/TwatVault.app

# Build for x86 and ARM
cargo build --release --target=aarch64-apple-darwin
cargo build --release --target=x86_64-apple-darwin

# Combine into a fat binary

lipo -create target/aarch64-apple-darwin/release/twatvault target/x86_64-apple-darwin/release/twatvault -output twatvault

# Perform Cargo bundle to create a macOS Bundle

cargo bundle --release

# Override bundle binary with the fat one

rm target/release/bundle/osx/TwatVault.app/Contents/MacOS/twatvault

mv ./twatvault target/release/bundle/osx/TwatVault.app/Contents/MacOS/TwatVault

# Tell the Info.plist or binary is capitalized

/usr/libexec/PlistBuddy -c "Set :CFBundleExecutable TwatVault" "target/release/bundle/osx/TwatVault.app/Contents/Info.plist"

# Create a zip file
cd target/release/bundle/osx/
/usr/bin/zip -5 -r ../../../twatvault.zip ./TwatVault.app
echo "Wrote zip file ../target/twatvault.zip"
