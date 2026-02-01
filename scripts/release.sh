#!/bin/bash

# Release script for Streamer
# Usage: ./scripts/release.sh [major|minor|patch|VERSION]
# Examples:
#   ./scripts/release.sh patch   # 1.0.0 -> 1.0.1
#   ./scripts/release.sh minor   # 1.0.0 -> 1.1.0
#   ./scripts/release.sh major   # 1.0.0 -> 2.0.0
#   ./scripts/release.sh 1.2.3   # Set specific version
#   ./scripts/release.sh         # Auto-increment patch

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version from tauri.conf.json
CURRENT_VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "\(.*\)".*/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo -e "${RED}Error: Could not determine current version${NC}"
    exit 1
fi

echo -e "${YELLOW}Current version: ${CURRENT_VERSION}${NC}"

# Parse current version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Determine new version based on argument
BUMP_TYPE=${1:-patch}

case $BUMP_TYPE in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        VERSION="${MAJOR}.${MINOR}.${PATCH}"
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        VERSION="${MAJOR}.${MINOR}.${PATCH}"
        ;;
    patch)
        PATCH=$((PATCH + 1))
        VERSION="${MAJOR}.${MINOR}.${PATCH}"
        ;;
    *)
        # Assume it's a specific version number
        if [[ $BUMP_TYPE =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            VERSION=$BUMP_TYPE
        else
            echo -e "${RED}Error: Invalid version or bump type: ${BUMP_TYPE}${NC}"
            echo "Usage: ./scripts/release.sh [major|minor|patch|X.Y.Z]"
            exit 1
        fi
        ;;
esac

if [ -z "$VERSION" ]; then
    echo -e "${RED}Error: Could not determine version${NC}"
    exit 1
fi

echo -e "${GREEN}📦 Building Streamer v${VERSION}${NC}"

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
    echo "Install with: brew install gh"
    exit 1
fi

# Check if logged in to gh
if ! gh auth status &> /dev/null; then
    echo -e "${RED}Error: Not logged in to GitHub CLI${NC}"
    echo "Run: gh auth login"
    exit 1
fi

# Update version in config files
echo -e "${YELLOW}Updating version to ${VERSION}...${NC}"
sed -i '' "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" src-tauri/tauri.conf.json
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" src-tauri/Cargo.toml

# Set up Android build environment
export JAVA_HOME=/opt/homebrew/Cellar/openjdk@17/17.0.18/libexec/openjdk.jdk/Contents/Home
export ANDROID_HOME=~/Android/sdk
export ANDROID_NDK_HOME=~/Android/sdk/ndk/27.0.12077973
export NDK_HOME=$ANDROID_NDK_HOME
TOOLCHAIN=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64
export PATH=$TOOLCHAIN/bin:$PATH
export CC_armv7_linux_androideabi=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang
export CXX_armv7_linux_androideabi=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang++
export AR_armv7_linux_androideabi=$TOOLCHAIN/bin/llvm-ar
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang

# Build release APK for Android
echo -e "${YELLOW}Building release APK...${NC}"
pnpm tauri android build --target armv7

APK_PATH="src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk"
SIGNED_APK="streamer-v${VERSION}.apk"
KEYSTORE="debug.keystore"
ANDROID_HOME=~/Android/sdk

if [ ! -f "$APK_PATH" ]; then
    echo -e "${RED}Error: APK not found at ${APK_PATH}${NC}"
    echo "Trying debug APK..."
    APK_PATH="src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk"
fi

if [ ! -f "$APK_PATH" ]; then
    echo -e "${RED}Error: No APK found${NC}"
    exit 1
fi

# Create debug keystore if it doesn't exist
if [ ! -f "$KEYSTORE" ]; then
    echo -e "${YELLOW}Creating debug keystore...${NC}"
    keytool -genkey -v -keystore "$KEYSTORE" -alias androiddebugkey \
        -keyalg RSA -keysize 2048 -validity 10000 \
        -storepass android -keypass android \
        -dname "CN=Debug, OU=Debug, O=Debug, L=Debug, ST=Debug, C=US"
fi

# Sign the APK
echo -e "${YELLOW}Signing APK...${NC}"
$ANDROID_HOME/build-tools/35.0.0/apksigner sign \
    --ks "$KEYSTORE" \
    --ks-pass pass:android \
    --key-pass pass:android \
    --out "$SIGNED_APK" \
    "$APK_PATH"

echo -e "${GREEN}APK signed: ${SIGNED_APK}${NC}"

# Commit version bump
echo -e "${YELLOW}Committing version bump...${NC}"
git add src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "Bump version to ${VERSION}" || true

# Create git tag
echo -e "${YELLOW}Creating git tag v${VERSION}...${NC}"
git tag -a "v${VERSION}" -m "Release v${VERSION}" 2>/dev/null || {
    echo -e "${YELLOW}Tag already exists, skipping...${NC}"
}

# Push to remote
echo -e "${YELLOW}Pushing to remote...${NC}"
git push origin main --tags 2>/dev/null || {
    echo -e "${YELLOW}Push failed - you may need to set up remote first${NC}"
    echo "Run: git remote add origin https://github.com/YOUR_USERNAME/streamer.git"
}

# Create GitHub release
echo -e "${YELLOW}Creating GitHub release...${NC}"

RELEASE_NOTES=$(cat <<EOF
## Streamer v${VERSION}

### What's New
- Stream movies from YTS on your Android TV
- Netflix-style browsing interface
- In-app video player with controls
- Subtitle support via SubDL
- Auto-update checker
- D-pad/remote navigation optimized

### Installation
1. Download the APK below
2. Enable "Install from unknown sources" on your Android TV
3. Install using a file manager or ADB:
   \`\`\`
   adb install streamer-v${VERSION}.apk
   \`\`\`

### Requirements
- Android 7.0+ (API 24)
- Internet connection
EOF
)

gh release create "v${VERSION}" \
    --title "Streamer v${VERSION}" \
    --notes "$RELEASE_NOTES" \
    "$SIGNED_APK" \
    2>/dev/null || {
    echo -e "${YELLOW}Release may already exist. Uploading asset...${NC}"
    gh release upload "v${VERSION}" "$SIGNED_APK" --clobber 2>/dev/null || true
}

# Clean up
rm -f "$SIGNED_APK"

echo ""
echo -e "${GREEN}✅ Release v${VERSION} created successfully!${NC}"
echo -e "View at: https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/releases/tag/v${VERSION}"
