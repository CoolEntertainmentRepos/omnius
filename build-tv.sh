#!/bin/bash
# Build and deploy Omnius to Mi Box (Android TV)
set -e

export JAVA_HOME=/opt/homebrew/Cellar/openjdk@17/17.0.18/libexec/openjdk.jdk/Contents/Home
export ANDROID_HOME=~/Android/sdk
export NDK_HOME=~/Android/sdk/ndk/28.0.13004108

MI_BOX="192.168.1.155:5555"
APK_PATH="src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk"

echo "==> Connecting to Mi Box..."
$ANDROID_HOME/platform-tools/adb connect $MI_BOX 2>/dev/null || true

echo "==> Building frontend..."
npm run build

echo "==> Building Rust for ARM..."
cd src-tauri
cargo build --target armv7-linux-androideabi --release --lib
cd ..

echo "==> Ensuring .so is in jniLibs..."
JNILIBS="src-tauri/gen/android/app/src/main/jniLibs/armeabi-v7a"
mkdir -p "$JNILIBS"
cp src-tauri/target/armv7-linux-androideabi/release/libstreamer_lib.so "$JNILIBS/"

echo "==> Building APK..."
src-tauri/gen/android/gradlew -p src-tauri/gen/android assembleUniversalDebug \
  -x rustBuildArm64Debug -x rustBuildArmDebug -x rustBuildX86Debug -x rustBuildX86_64Debug

echo "==> Installing on Mi Box..."
$ANDROID_HOME/platform-tools/adb -s $MI_BOX install -r "$APK_PATH"

echo "==> Launching..."
$ANDROID_HOME/platform-tools/adb -s $MI_BOX shell am start -n lol.omnius.tv/.MainActivity

echo "==> Done!"
