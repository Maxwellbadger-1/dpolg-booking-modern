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

# Load signing key from file
if [ -f "src-tauri/dpolg-signing.key" ]; then
  export TAURI_SIGNING_PRIVATE_KEY=$(cat src-tauri/dpolg-signing.key)
  export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
else
  echo "❌ ERROR: Signing key not found at src-tauri/dpolg-signing.key!"
  exit 1
fi

npm run tauri build
echo "✅ Build completed!"
echo ""

# Step 5: Prepare release data
echo "📦 Step 5/7: Preparing GitHub release data..."
cat > release-data.json << EOF
{
  "tag_name": "v${VERSION}",
  "name": "Stiftung der DPolG Buchungssystem v${VERSION}",
  "body": "## 🎉 Änderungen in v${VERSION}\n\n- ✅ [Beschreibung der Änderungen hier einfügen]\n\n## 📥 Installation\n**Windows:** Laden Sie die \`-setup.exe\` Datei herunter und installieren Sie die App.\n\n## 🔄 Auto-Update\nWenn Sie bereits eine ältere Version installiert haben, wird automatisch ein Update-Dialog angezeigt.",
  "draft": false,
  "prerelease": false
}
EOF
echo "✅ Release data prepared!"
echo ""

# Step 6: Create GitHub release
echo "📤 Step 6/7: Creating GitHub release..."

# Load GitHub token from .github-token file if not already set
if [ -z "$GITHUB_TOKEN" ]; then
  if [ -f ".github-token" ]; then
    GITHUB_TOKEN=$(cat .github-token)
  else
    echo "❌ ERROR: GITHUB_TOKEN not set and .github-token file not found!"
    echo "Create .github-token file with your GitHub token or set GITHUB_TOKEN environment variable."
    exit 1
  fi
fi

RELEASE_RESPONSE=$(curl -s -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${GITHUB_TOKEN}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  -H "Content-Type: application/json" \
  https://api.github.com/repos/Maxwellbadger-1/dpolg-booking-modern/releases \
  -d @release-data.json)

RELEASE_ID=$(echo "$RELEASE_RESPONSE" | grep -o '"id": [0-9]*' | head -1 | grep -o '[0-9]*')
echo "✅ Release created! ID: ${RELEASE_ID}"
echo ""

# Step 7: Upload files
echo "⬆️  Step 7/7: Uploading files to GitHub..."
cd src-tauri/target/release/bundle/nsis

# Upload NSIS installer (.exe) file
echo "  Uploading NSIS installer file..."
curl -s -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${GITHUB_TOKEN}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  -H "Content-Type: application/octet-stream" \
  "https://uploads.github.com/repos/Maxwellbadger-1/dpolg-booking-modern/releases/${RELEASE_ID}/assets?name=Stiftung.der.DPolG.Buchungssystem_${VERSION}_x64-setup.exe" \
  --data-binary "@Stiftung der DPolG Buchungssystem_${VERSION}_x64-setup.exe" > /dev/null

echo "  ✅ Installer uploaded!"

# Upload .sig file
echo "  Uploading .sig file..."
curl -s -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${GITHUB_TOKEN}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  -H "Content-Type: application/octet-stream" \
  "https://uploads.github.com/repos/Maxwellbadger-1/dpolg-booking-modern/releases/${RELEASE_ID}/assets?name=Stiftung.der.DPolG.Buchungssystem_${VERSION}_x64-setup.exe.sig" \
  --data-binary "@Stiftung der DPolG Buchungssystem_${VERSION}_x64-setup.exe.sig" > /dev/null

echo "  ✅ .sig uploaded!"

# Upload latest.json for Tauri updater
echo "  Creating latest.json for Tauri updater..."
SIGNATURE=$(cat "Stiftung der DPolG Buchungssystem_${VERSION}_x64-setup.exe.sig")
cat > latest.json << EOF
{
  "version": "${VERSION}",
  "notes": "Release v${VERSION}",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "windows-x86_64": {
      "signature": "${SIGNATURE}",
      "url": "https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/download/v${VERSION}/Stiftung.der.DPolG.Buchungssystem_${VERSION}_x64-setup.exe"
    }
  }
}
EOF

echo "  Uploading latest.json..."
curl -s -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${GITHUB_TOKEN}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  -H "Content-Type: application/json" \
  "https://uploads.github.com/repos/Maxwellbadger-1/dpolg-booking-modern/releases/${RELEASE_ID}/assets?name=latest.json" \
  --data-binary "@latest.json" > /dev/null

echo "  ✅ latest.json uploaded!"
echo ""

# Cleanup
cd ../../../../..
rm release-data.json latest.json 2>/dev/null || true

echo "🎉 Release v${VERSION} erfolgreich veröffentlicht!"
echo ""
echo "🔗 Release URL: https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/tag/v${VERSION}"
echo ""
echo "🧪 Nächster Schritt: Auto-Update testen"
echo "   1. Öffnen Sie die installierte App (ältere Version)"
echo "   2. Update-Dialog sollte erscheinen"
echo "   3. Klicken Sie auf 'Ja' zum Updaten"
echo ""
echo "✅ Quick Release Script completed!"
