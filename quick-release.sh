#!/bin/bash
# Quick Release Script for DPolG Booking System
# Usage: ./quick-release.sh 1.7.5

set -e  # Exit on error

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "❌ ERROR: Version number required!"
  echo "Usage: ./quick-release.sh 1.7.5"
  exit 1
fi

echo "🚀 Starting Quick Release for v${VERSION}"
echo ""

# Step 1: Update version in all files
echo "📝 Step 1/5: Updating version numbers..."
sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" package.json
sed -i "s/version = \".*\"/version = \"${VERSION}\"/" src-tauri/Cargo.toml
sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" src-tauri/tauri.conf.json
echo "✅ Version updated to ${VERSION}"
echo ""

# Step 2: Commit version bump
echo "📝 Step 2/5: Committing version bump..."
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: Bump version to ${VERSION}"
echo "✅ Version committed"
echo ""

# Step 3: Create and push tag
echo "📝 Step 3/5: Creating git tag..."
git tag -a "v${VERSION}" -m "Release v${VERSION}"
git push && git push --tags
echo "✅ Tag v${VERSION} created and pushed"
echo ""

# Step 4: Build with signing
echo "🏗️  Step 4/5: Building release binaries (this takes ~5-10 minutes)..."
export TAURI_SIGNING_PRIVATE_KEY="dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5YlJWSmYwd3pwYXNSLzh5aWlJdkR0S3ZWRFBVY3JTK3RjL2I5R042YmNzY0FBQkFBQUFBQUFBQUFBQUlBQUFBQUV6L0ZWcE56UzFFVnlpYUpHKy80dlVOYzlERmNVWjdMTVFmSHAzQ0NiaU42OUtzaVQ0SFlPVFVIMXl2NVE0bHVpRUoxU3YzSm8rTThCS0s2L200UzF0SHB1bXpZeDgyV0RiajhhUDdjM3RJZlVuNkVMcGlJUFJNWnFrdmFYZkpzdXFzZndkSk0ybjQ9Cg=="
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
npm run tauri build
echo "✅ Build completed!"
echo ""

# Step 5: Show next steps
echo "📦 Step 5/5: Files ready for upload!"
echo ""
echo "Files location: src-tauri/target/release/bundle/msi/"
ls -lh "src-tauri/target/release/bundle/msi/" | grep "\.msi"
echo ""
echo "🎯 NEXT STEPS:"
echo "1. Go to: https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/new"
echo "2. Tag: v${VERSION}"
echo "3. Upload BOTH files from msi/ folder:"
echo "   - *.msi (Installer + Update Package)"
echo "   - *.msi.sig (Signatur)"
echo "4. Click 'Publish release' (NOT Draft!)"
echo ""
echo "⚠️  Tauri 2 Note: No .msi.zip needed - the .msi is used directly as update package!"
echo ""
echo "✅ Quick Release Script completed!"
