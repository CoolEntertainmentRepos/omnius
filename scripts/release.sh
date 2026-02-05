#!/usr/bin/env bash

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
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RELEASE_DIR="$PROJECT_DIR/releases"

# Get current version from tauri.conf.json
CURRENT_VERSION=$(grep '"version"' "$PROJECT_DIR/src-tauri/tauri.conf.json" | head -1 | sed 's/.*: "\(.*\)".*/\1/')

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

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Building Streamer v${VERSION}${NC}"
echo -e "${GREEN}========================================${NC}"

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
sed -i '' "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" "$PROJECT_DIR/src-tauri/tauri.conf.json"
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" "$PROJECT_DIR/src-tauri/Cargo.toml"

# Create releases directory
mkdir -p "$RELEASE_DIR"
RELEASE_FILES=()

# ============================================
# Android TV Builds
# ============================================
echo -e "\n${BLUE}=== Building Android TV APKs ===${NC}"

# Set up Android build environment
export JAVA_HOME=/opt/homebrew/Cellar/openjdk@17/17.0.18/libexec/openjdk.jdk/Contents/Home
export ANDROID_HOME=~/Library/Android/sdk
export ANDROID_NDK_HOME=~/Library/Android/sdk/ndk/28.2.13676358
export NDK_HOME=$ANDROID_NDK_HOME
TOOLCHAIN=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64
export PATH=$TOOLCHAIN/bin:$PATH

# ARM toolchain (32-bit - for older devices like Mi Box)
export CC_armv7_linux_androideabi=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang
export CXX_armv7_linux_androideabi=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang++
export AR_armv7_linux_androideabi=$TOOLCHAIN/bin/llvm-ar
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang

# ARM64 toolchain (64-bit - for newer devices)
export CC_aarch64_linux_android=$TOOLCHAIN/bin/aarch64-linux-android24-clang
export CXX_aarch64_linux_android=$TOOLCHAIN/bin/aarch64-linux-android24-clang++
export AR_aarch64_linux_android=$TOOLCHAIN/bin/llvm-ar
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$TOOLCHAIN/bin/aarch64-linux-android24-clang

KEYSTORE="$PROJECT_DIR/debug.keystore"

# Create debug keystore if it doesn't exist
if [ ! -f "$KEYSTORE" ]; then
    echo -e "${YELLOW}Creating debug keystore...${NC}"
    keytool -genkey -v -keystore "$KEYSTORE" -alias androiddebugkey \
        -keyalg RSA -keysize 2048 -validity 10000 \
        -storepass android -keypass android \
        -dname "CN=Debug, OU=Debug, O=Debug, L=Debug, ST=Debug, C=US"
fi

# Build and sign ARM (32-bit)
echo -e "${YELLOW}Building Android TV (ARM 32-bit)...${NC}"
cd "$PROJECT_DIR"
pnpm tauri android build --target armv7 2>&1 | tail -5

APK_ARM="$PROJECT_DIR/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk"
if [ -f "$APK_ARM" ]; then
    SIGNED_ARM="$RELEASE_DIR/Streamer-v${VERSION}-android-tv-arm.apk"
    $ANDROID_HOME/build-tools/35.0.0/apksigner sign \
        --ks "$KEYSTORE" --ks-pass pass:android --key-pass pass:android \
        --out "$SIGNED_ARM" "$APK_ARM"
    echo -e "${GREEN}✓ Built: Streamer-v${VERSION}-android-tv-arm.apk${NC}"
    RELEASE_FILES+=("$SIGNED_ARM")
fi

# Build and sign ARM64 (64-bit)
echo -e "${YELLOW}Building Android TV (ARM 64-bit)...${NC}"
pnpm tauri android build --target aarch64 2>&1 | tail -5

APK_ARM64="$PROJECT_DIR/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk"
if [ -f "$APK_ARM64" ]; then
    SIGNED_ARM64="$RELEASE_DIR/Streamer-v${VERSION}-android-tv-arm64.apk"
    $ANDROID_HOME/build-tools/35.0.0/apksigner sign \
        --ks "$KEYSTORE" --ks-pass pass:android --key-pass pass:android \
        --out "$SIGNED_ARM64" "$APK_ARM64"
    echo -e "${GREEN}✓ Built: Streamer-v${VERSION}-android-tv-arm64.apk${NC}"
    RELEASE_FILES+=("$SIGNED_ARM64")
fi

# ============================================
# Desktop Builds (with Tauri Updater signing)
# ============================================
echo -e "\n${BLUE}=== Building Desktop Apps ===${NC}"

# Set up signing for Tauri Updater
export TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/streamer.key)
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""

GITHUB_REPO_URL="https://github.com/flakerim/Streamer/releases/download/v${VERSION}"

# Manifest platform entries (will be built up as we go)
DARWIN_AARCH64_ENTRY=""
DARWIN_X86_64_ENTRY=""

# macOS (Apple Silicon)
echo -e "${YELLOW}Building macOS (Apple Silicon)...${NC}"
pnpm tauri build --target aarch64-apple-darwin 2>&1 | tail -5

DMG_ARM64=$(find "$PROJECT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/dmg" -name "*.dmg" 2>/dev/null | head -1)
TARGZ_ARM64=$(find "$PROJECT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/macos" -name "*.tar.gz" 2>/dev/null | head -1)
SIG_ARM64=$(find "$PROJECT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/macos" -name "*.tar.gz.sig" 2>/dev/null | head -1)

if [ -f "$DMG_ARM64" ]; then
    MACOS_ARM64="$RELEASE_DIR/Streamer-v${VERSION}-macos-arm64.dmg"
    cp "$DMG_ARM64" "$MACOS_ARM64"
    echo -e "${GREEN}✓ Built: Streamer-v${VERSION}-macos-arm64.dmg${NC}"
    RELEASE_FILES+=("$MACOS_ARM64")
fi

if [ -f "$TARGZ_ARM64" ] && [ -f "$SIG_ARM64" ]; then
    MACOS_ARM64_UPDATE="$RELEASE_DIR/Streamer-v${VERSION}-macos-arm64.app.tar.gz"
    cp "$TARGZ_ARM64" "$MACOS_ARM64_UPDATE"
    RELEASE_FILES+=("$MACOS_ARM64_UPDATE")
    SIG_CONTENT=$(cat "$SIG_ARM64")
    DARWIN_AARCH64_ENTRY="\"darwin-aarch64\":{\"signature\":\"$SIG_CONTENT\",\"url\":\"$GITHUB_REPO_URL/Streamer-v${VERSION}-macos-arm64.app.tar.gz\"}"
    echo -e "${GREEN}✓ Built updater: Streamer-v${VERSION}-macos-arm64.app.tar.gz${NC}"
fi

# macOS (Intel)
echo -e "${YELLOW}Building macOS (Intel x64)...${NC}"
pnpm tauri build --target x86_64-apple-darwin 2>&1 | tail -5

DMG_X64=$(find "$PROJECT_DIR/src-tauri/target/x86_64-apple-darwin/release/bundle/dmg" -name "*.dmg" 2>/dev/null | head -1)
TARGZ_X64=$(find "$PROJECT_DIR/src-tauri/target/x86_64-apple-darwin/release/bundle/macos" -name "*.tar.gz" 2>/dev/null | head -1)
SIG_X64=$(find "$PROJECT_DIR/src-tauri/target/x86_64-apple-darwin/release/bundle/macos" -name "*.tar.gz.sig" 2>/dev/null | head -1)

if [ -f "$DMG_X64" ]; then
    MACOS_X64="$RELEASE_DIR/Streamer-v${VERSION}-macos-x64.dmg"
    cp "$DMG_X64" "$MACOS_X64"
    echo -e "${GREEN}✓ Built: Streamer-v${VERSION}-macos-x64.dmg${NC}"
    RELEASE_FILES+=("$MACOS_X64")
fi

if [ -f "$TARGZ_X64" ] && [ -f "$SIG_X64" ]; then
    MACOS_X64_UPDATE="$RELEASE_DIR/Streamer-v${VERSION}-macos-x64.app.tar.gz"
    cp "$TARGZ_X64" "$MACOS_X64_UPDATE"
    RELEASE_FILES+=("$MACOS_X64_UPDATE")
    SIG_CONTENT=$(cat "$SIG_X64")
    DARWIN_X86_64_ENTRY="\"darwin-x86_64\":{\"signature\":\"$SIG_CONTENT\",\"url\":\"$GITHUB_REPO_URL/Streamer-v${VERSION}-macos-x64.app.tar.gz\"}"
    echo -e "${GREEN}✓ Built updater: Streamer-v${VERSION}-macos-x64.app.tar.gz${NC}"
fi

# Generate latest.json manifest for Tauri Updater
echo -e "${YELLOW}Generating updater manifest...${NC}"
PUB_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Build platforms JSON
PLATFORMS_JSON=""
if [ -n "$DARWIN_AARCH64_ENTRY" ]; then
    PLATFORMS_JSON="$DARWIN_AARCH64_ENTRY"
fi
if [ -n "$DARWIN_X86_64_ENTRY" ]; then
    if [ -n "$PLATFORMS_JSON" ]; then
        PLATFORMS_JSON="$PLATFORMS_JSON,$DARWIN_X86_64_ENTRY"
    else
        PLATFORMS_JSON="$DARWIN_X86_64_ENTRY"
    fi
fi

if [ -n "$PLATFORMS_JSON" ]; then
    cat > "$RELEASE_DIR/latest.json" <<EOF
{
  "version": "${VERSION}",
  "notes": "Streamer v${VERSION}",
  "pub_date": "${PUB_DATE}",
  "platforms": {
    $PLATFORMS_JSON
  }
}
EOF
    RELEASE_FILES+=("$RELEASE_DIR/latest.json")
    echo -e "${GREEN}✓ Generated: latest.json${NC}"
else
    echo -e "${YELLOW}⚠ No desktop builds, skipping latest.json${NC}"
fi

# ============================================
# Git & GitHub Release
# ============================================
echo -e "\n${BLUE}=== Creating Release ===${NC}"

# Commit version bump
echo -e "${YELLOW}Committing version bump...${NC}"
git add "$PROJECT_DIR/src-tauri/tauri.conf.json" "$PROJECT_DIR/src-tauri/Cargo.toml"
git commit -m "Release v${VERSION}" || true

# Create git tag
echo -e "${YELLOW}Creating git tag v${VERSION}...${NC}"
git tag -a "v${VERSION}" -m "Release v${VERSION}" 2>/dev/null || {
    echo -e "${YELLOW}Tag already exists, deleting and recreating...${NC}"
    git tag -d "v${VERSION}" 2>/dev/null || true
    git push origin --delete "v${VERSION}" 2>/dev/null || true
    git tag -a "v${VERSION}" -m "Release v${VERSION}"
}

# Push to remote
echo -e "${YELLOW}Pushing to remote...${NC}"
git push origin main --tags

# Create GitHub release
echo -e "${YELLOW}Creating GitHub release...${NC}"

RELEASE_NOTES=$(cat <<EOF
## Streamer v${VERSION}

### Downloads

| Platform | Architecture | File |
|----------|--------------|------|
| Android TV | ARM (32-bit) | \`Streamer-v${VERSION}-android-tv-arm.apk\` |
| Android TV | ARM64 (64-bit) | \`Streamer-v${VERSION}-android-tv-arm64.apk\` |
| macOS | Apple Silicon | \`Streamer-v${VERSION}-macos-arm64.dmg\` |
| macOS | Intel | \`Streamer-v${VERSION}-macos-x64.dmg\` |

### Android TV Installation

1. Download the APK for your device:
   - **Mi Box, older Android TV**: Use \`arm\` version
   - **Newer Android TV, Shield**: Use \`arm64\` version
2. Enable "Install from unknown sources" in Settings
3. Install via file manager or ADB:
   \`\`\`bash
   adb install Streamer-v${VERSION}-android-tv-arm64.apk
   \`\`\`

### macOS Installation

1. Download the DMG for your Mac
2. Open the DMG and drag Streamer to Applications
3. First launch: Right-click > Open (to bypass Gatekeeper)

### Features

- Stream movies from YTS on Android TV
- Netflix-style browsing with D-pad navigation
- In-app video player with controls
- Subtitle support (60+ languages) via SubDL
- Auto-load subtitles based on language preference
- Auto-update checker

### Requirements

- **Android TV**: Android 7.0+ (API 24)
- **macOS**: macOS 10.15+
EOF
)

# Delete existing release if it exists
gh release delete "v${VERSION}" --yes 2>/dev/null || true

# Create new release with all files
gh release create "v${VERSION}" \
    --title "Streamer v${VERSION}" \
    --notes "$RELEASE_NOTES" \
    "${RELEASE_FILES[@]}"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Release v${VERSION} complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Files built:"
for f in "${RELEASE_FILES[@]}"; do
    echo -e "  ${BLUE}$(basename "$f")${NC}"
done
echo ""
echo -e "View at: ${BLUE}https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/releases/tag/v${VERSION}${NC}"
