#!/bin/bash

# Setup script for Android TV configuration
# Run this after: pnpm tauri android init

set -e

MANIFEST_PATH="src-tauri/gen/android/app/src/main/AndroidManifest.xml"
STRINGS_PATH="src-tauri/gen/android/app/src/main/res/values/strings.xml"

echo "🔧 Configuring Streamer for Android TV..."

# Check if manifest exists
if [ ! -f "$MANIFEST_PATH" ]; then
    echo "❌ AndroidManifest.xml not found!"
    echo "   Run 'pnpm tauri android init' first."
    exit 1
fi

# Backup original manifest
cp "$MANIFEST_PATH" "${MANIFEST_PATH}.backup"

# Create the TV-optimized AndroidManifest.xml
cat > "$MANIFEST_PATH" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

    <!-- Permissions -->
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
    <uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="28" />
    <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="32" />

    <!-- TV Features - not required so app works on both TV and mobile -->
    <uses-feature android:name="android.software.leanback" android:required="false" />
    <uses-feature android:name="android.hardware.touchscreen" android:required="false" />

    <application
        android:label="@string/app_name"
        android:icon="@mipmap/ic_launcher"
        android:banner="@mipmap/ic_launcher"
        android:theme="@style/Theme.Streamer"
        android:allowBackup="true"
        android:supportsRtl="true"
        android:usesCleartextTraffic="true"
        android:hardwareAccelerated="true"
        android:largeHeap="true">

        <activity
            android:name=".MainActivity"
            android:label="@string/app_name"
            android:exported="true"
            android:launchMode="singleTop"
            android:configChanges="orientation|keyboardHidden|keyboard|screenSize|smallestScreenSize|locale|layoutDirection|fontScale|screenLayout|density|uiMode"
            android:theme="@style/Theme.Streamer"
            android:screenOrientation="landscape">

            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <!-- Leanback launcher for Android TV -->
                <category android:name="android.intent.category.LEANBACK_LAUNCHER" />
                <!-- Standard launcher for phones/tablets -->
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
EOF

echo "✅ AndroidManifest.xml configured for TV"

# Create/update strings.xml
mkdir -p "$(dirname $STRINGS_PATH)"
cat > "$STRINGS_PATH" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">Streamer</string>
</resources>
EOF

echo "✅ strings.xml updated"

# Create styles.xml for fullscreen theme
STYLES_PATH="src-tauri/gen/android/app/src/main/res/values/styles.xml"
cat > "$STYLES_PATH" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <style name="Theme.Streamer" parent="android:Theme.Material.NoActionBar">
        <item name="android:windowFullscreen">true</item>
        <item name="android:windowContentOverlay">@null</item>
        <item name="android:windowBackground">@android:color/black</item>
        <item name="android:colorBackground">@android:color/black</item>
    </style>
</resources>
EOF

echo "✅ Theme configured (fullscreen, dark background)"

# Update build.gradle for TV
BUILD_GRADLE="src-tauri/gen/android/app/build.gradle.kts"
if [ -f "$BUILD_GRADLE" ]; then
    # Check if minSdk is already set correctly
    if grep -q "minSdk" "$BUILD_GRADLE"; then
        sed -i '' 's/minSdk = [0-9]*/minSdk = 24/' "$BUILD_GRADLE" 2>/dev/null || true
        echo "✅ minSdk set to 24 (Android 7.0+)"
    fi
fi

echo ""
echo "🎉 Android TV setup complete!"
echo ""
echo "Next steps:"
echo "  1. Build debug APK:   pnpm tauri android build --debug"
echo "  2. Build release APK: pnpm tauri android build --release"
echo ""
echo "APK location: src-tauri/gen/android/app/build/outputs/apk/"
echo ""
