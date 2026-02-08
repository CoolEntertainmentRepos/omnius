# Omnius TV - Build Guide

## Prerequisites

- **Node.js** (v18+)
- **Rust** (via rustup)
- **Android SDK**: `~/Android/sdk`
- **Android NDK**: `~/Android/sdk/ndk/`
- **Java 17**: `/opt/homebrew/Cellar/openjdk@17/17.0.18/libexec/openjdk.jdk/Contents/Home`
- **ADB** for deploying to TV

## Environment

```bash
export JAVA_HOME=/opt/homebrew/Cellar/openjdk@17/17.0.18/libexec/openjdk.jdk/Contents/Home
export ANDROID_HOME=~/Android/sdk
```

## Mi Box Info

- Architecture: **armv7**
- ADB address: `192.168.1.155:5555`
- Package name: `lol.omnius.tv`

---

## Full Build (Frontend + Rust + APK)

Use this when you need everything rebuilt from scratch (new native code, first build, etc.):

```bash
# Build for Mi Box (armv7)
npm run tauri android build -- --target armv7 --debug
```

This runs: frontend build -> Rust cross-compile -> Gradle APK build

## Frontend-Only Build (Fast)

Use this when you ONLY changed Svelte/JS/CSS code (no Rust changes).

**IMPORTANT**: The fast Gradle-only build (`./gradlew assembleUniversalDebug -x rust*`) does NOT copy new frontend files into the APK assets. You MUST use `npm run tauri android build` to properly sync frontend assets.

```bash
# Correct way - always use tauri build
npm run tauri android build -- --target armv7 --debug
```

The previous "fast build" shortcut skips the Tauri asset copy step and will produce an APK with stale/old JS files:
```bash
# DO NOT USE for frontend changes - assets won't update!
# ./gradlew assembleUniversalDebug -x rustBuild*
```

## Install on Mi Box

```bash
# Connect to Mi Box
~/Android/sdk/platform-tools/adb connect 192.168.1.155:5555

# Install APK (overwrite existing)
~/Android/sdk/platform-tools/adb -s 192.168.1.155:5555 install -r \
  src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

## Clean Install (clear all caches)

Use when you suspect WebView cache or old data is stuck:

```bash
# Uninstall first (clears app data + WebView cache)
~/Android/sdk/platform-tools/adb -s 192.168.1.155:5555 uninstall lol.omnius.tv

# Then install fresh
~/Android/sdk/platform-tools/adb -s 192.168.1.155:5555 install \
  src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

## Full Clean Rebuild

Nuclear option - clean everything and rebuild:

```bash
# Clean frontend
rm -rf build .svelte-kit

# Clean Gradle
cd src-tauri/gen/android && ./gradlew clean && cd ../../..

# Full rebuild
npm run tauri android build -- --target armv7 --debug
```

## Dev Server (Browser Testing)

```bash
npm run dev -- --port 3333
```

Opens at `http://localhost:3333`. Note: HLS streams won't play in Chrome without hls.js (Safari works natively).

## Desktop (Tauri Dev)

```bash
npm run tauri dev
```
