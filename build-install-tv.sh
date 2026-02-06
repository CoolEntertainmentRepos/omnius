#!/bin/bash
# Build and install on Mi Box using Tauri CLI (proper build that embeds frontend in Rust binary)
# Usage: ./build-install-tv.sh

set -e

ANDROID_HOME=~/Android/sdk
JAVA_HOME=/opt/homebrew/Cellar/openjdk@17/17.0.18/libexec/openjdk.jdk/Contents/Home
MI_BOX="192.168.1.155:5555"
APK_PATH="src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk"
PKG="lol.omnius.tv"
ADB="$ANDROID_HOME/platform-tools/adb -s $MI_BOX"

export ANDROID_HOME JAVA_HOME

cd "$(dirname "$0")"

echo "=== Step 1: Build with Tauri CLI (frontend + Rust + APK) ==="
npm run tauri android build -- --debug --target armv7

echo "=== Step 2: Install on Mi Box ($MI_BOX) ==="
$ADB install -r "$APK_PATH"

echo "=== Step 3: Clear app cache + launch ==="
$ADB shell pm clear "$PKG"
$ADB shell am start -n "$PKG/.MainActivity"

echo "=== Done! ==="
