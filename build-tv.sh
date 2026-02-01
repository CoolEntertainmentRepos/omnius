#!/bin/bash
set -e

# Android TV Build Script for Streamer

export JAVA_HOME=/opt/homebrew/Cellar/openjdk@17/17.0.18/libexec/openjdk.jdk/Contents/Home
export ANDROID_HOME=~/Android/sdk
export ANDROID_NDK_HOME=~/Android/sdk/ndk/27.0.12077973
export NDK_HOME=$ANDROID_NDK_HOME

# Set up toolchain for ARM cross-compilation
TOOLCHAIN=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64
export PATH=$TOOLCHAIN/bin:$PATH
export CC_armv7_linux_androideabi=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang
export CXX_armv7_linux_androideabi=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang++
export AR_armv7_linux_androideabi=$TOOLCHAIN/bin/llvm-ar
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
KEYSTORE="$SCRIPT_DIR/debug.keystore"
UNSIGNED_APK="$SCRIPT_DIR/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk"
SIGNED_APK="$SCRIPT_DIR/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-signed.apk"

echo "=== Building Android APK with Tauri ==="
pnpm tauri android build --target armv7

echo "=== Signing APK ==="
$ANDROID_HOME/build-tools/35.0.0/apksigner sign \
  --ks "$KEYSTORE" \
  --ks-pass pass:android \
  --key-pass pass:android \
  --out "$SIGNED_APK" \
  "$UNSIGNED_APK"

echo "=== Installing on Mi Box ==="
~/Android/sdk/platform-tools/adb -s 192.168.1.155:5555 install -r "$SIGNED_APK"

echo "=== Done! ==="
