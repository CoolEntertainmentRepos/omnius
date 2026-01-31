# Building Streamer for Android TV

## Prerequisites

### 1. Install Java JDK 17
```bash
brew install openjdk@17

# Add to ~/.zshrc
export JAVA_HOME="/opt/homebrew/opt/openjdk@17"
export PATH="$JAVA_HOME/bin:$PATH"
```

### 2. Install Android Studio
Download from: https://developer.android.com/studio

In SDK Manager, install:
- Android SDK Platform 34 (or latest)
- Android SDK Build-Tools 34
- NDK (Side by side) - latest version
- Android SDK Command-line Tools

### 3. Set Environment Variables
Add to `~/.zshrc`:
```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk 2>/dev/null | sort -V | tail -1)"
export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
```

Then reload: `source ~/.zshrc`

### 4. Add Rust Android Targets
```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

## Build Steps

### Step 1: Initialize Android Project
```bash
cd /Users/flakerim/Dev/Streamer
pnpm tauri android init
```

### Step 2: Configure for Android TV
```bash
./scripts/setup-android-tv.sh
```

### Step 3: Build Debug APK (for testing)
```bash
pnpm tauri android build --debug
```

### Step 4: Build Release APK (for distribution)
```bash
pnpm tauri android build --release
```

## APK Location
- Debug: `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`
- Release: `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`

## Install on Mi Box

### Via ADB
```bash
# Enable Developer Options on Mi Box (Settings > About > Build number x7)
# Enable ADB debugging in Developer Options
# Find Mi Box IP in Settings > Network

adb connect <MI_BOX_IP>:5555
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

### Via USB
Copy APK to USB drive, plug into Mi Box, use file manager to install.

## Signing Release APK

For Play Store or sideloading signed APK:

```bash
# Generate keystore (one time)
keytool -genkey -v -keystore streamer.keystore -alias streamer -keyalg RSA -keysize 2048 -validity 10000

# Sign APK
jarsigner -verbose -sigalg SHA256withRSA -digestalg SHA-256 -keystore streamer.keystore app-universal-release-unsigned.apk streamer

# Align APK
zipalign -v 4 app-universal-release-unsigned.apk streamer-release.apk
```

## TV Remote Controls
The app is configured for D-pad navigation:
- Arrow keys: Navigate
- Enter/OK: Select
- Back: Go back
- Menu: Options (if implemented)

## Troubleshooting

### "SDK location not found"
Set ANDROID_HOME environment variable correctly.

### NDK not found
Install NDK via Android Studio SDK Manager.

### Build fails with native code errors
Make sure all Rust Android targets are installed:
```bash
rustup target list | grep android
```
