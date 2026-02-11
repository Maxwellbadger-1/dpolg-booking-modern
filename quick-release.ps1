# Quick Release Script for DPolG Booking System (Windows PowerShell)
# Usage: .\quick-release.ps1 -Version "1.8.0"

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "Starting Quick Release for v$Version" -ForegroundColor Cyan
Write-Host ""

# Paths
$ProjectRoot = $PSScriptRoot
$PackageJson = Join-Path $ProjectRoot "package.json"
$CargoToml = Join-Path $ProjectRoot "src-tauri\Cargo.toml"
$TauriConf = Join-Path $ProjectRoot "src-tauri\tauri.conf.json"
$SigningKey = Join-Path $ProjectRoot "src-tauri\dpolg-signing.key"
$TokenFile = Join-Path $ProjectRoot ".github-token"

# Step 1: Update version in all files
Write-Host "Step 1/7: Updating version numbers..." -ForegroundColor Yellow

# Update package.json
$packageContent = Get-Content $PackageJson -Raw
$packageContent = $packageContent -replace '"version": "[^"]*"', ('"version": "' + $Version + '"')
Set-Content -Path $PackageJson -Value $packageContent -NoNewline

# Update Cargo.toml (only first occurrence)
$cargoContent = Get-Content $CargoToml -Raw
$cargoContent = $cargoContent -replace '(?m)^version = "[^"]*"', ('version = "' + $Version + '"')
Set-Content -Path $CargoToml -Value $cargoContent -NoNewline

# Update tauri.conf.json
$tauriContent = Get-Content $TauriConf -Raw
$tauriContent = $tauriContent -replace '"version": "[^"]*"', ('"version": "' + $Version + '"')
Set-Content -Path $TauriConf -Value $tauriContent -NoNewline

Write-Host "Version updated to $Version" -ForegroundColor Green
Write-Host ""

# Step 2: Commit version bump
Write-Host "Step 2/7: Committing version bump..." -ForegroundColor Yellow
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: Bump version to $Version"
Write-Host "Version committed" -ForegroundColor Green
Write-Host ""

# Step 3: Create and push tag
Write-Host "Step 3/7: Creating git tag..." -ForegroundColor Yellow
git tag -a "v$Version" -m "Release v$Version"
git push
git push --tags
Write-Host "Tag v$Version created and pushed" -ForegroundColor Green
Write-Host ""

# Step 4: Build with signing
Write-Host "Step 4/7: Building release binaries (this takes ~5-10 minutes)..." -ForegroundColor Yellow

if (-not (Test-Path $SigningKey)) {
    Write-Host "ERROR: Signing key not found at $SigningKey!" -ForegroundColor Red
    exit 1
}

$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $SigningKey -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "dpolg2025"

npm run tauri build

Write-Host "Build completed!" -ForegroundColor Green
Write-Host ""

# Step 5: Load GitHub Token
Write-Host "Step 5/7: Loading GitHub token..." -ForegroundColor Yellow

if (-not (Test-Path $TokenFile)) {
    Write-Host "ERROR: .github-token file not found!" -ForegroundColor Red
    exit 1
}

$GithubToken = (Get-Content $TokenFile -Raw).Trim()
$Headers = @{
    "Accept" = "application/vnd.github+json"
    "Authorization" = "Bearer $GithubToken"
    "X-GitHub-Api-Version" = "2022-11-28"
}
$Repo = "Maxwellbadger-1/dpolg-booking-modern"

Write-Host "Token loaded" -ForegroundColor Green
Write-Host ""

# Step 6: Create GitHub release
Write-Host "Step 6/7: Creating GitHub release..." -ForegroundColor Yellow

$ReleaseBody = @{
    tag_name = "v$Version"
    name = "Stiftung der DPolG Buchungssystem v$Version"
    body = "## Aenderungen in v$Version`n`n- Undo/Redo System komplett implementiert`n- DevTools ein/ausschaltbar in Einstellungen`n- Windows PowerShell Scripts hinzugefuegt`n`n## Installation`n**Windows:** Laden Sie die -setup.exe Datei herunter und installieren Sie die App.`n`n## Auto-Update`nWenn Sie bereits eine aeltere Version installiert haben, wird automatisch ein Update-Dialog angezeigt."
    draft = $false
    prerelease = $false
} | ConvertTo-Json

$ReleaseResponse = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases" -Method Post -Headers $Headers -Body $ReleaseBody -ContentType "application/json"
$ReleaseId = $ReleaseResponse.id

Write-Host "Release created! ID: $ReleaseId" -ForegroundColor Green
Write-Host ""

# Step 7: Upload files
Write-Host "Step 7/7: Uploading files to GitHub..." -ForegroundColor Yellow

$NsisDir = Join-Path $ProjectRoot "src-tauri\target\release\bundle\nsis"
$ExeFile = Join-Path $NsisDir "Stiftung der DPolG Buchungssystem_${Version}_x64-setup.exe"
$SigFile = Join-Path $NsisDir "Stiftung der DPolG Buchungssystem_${Version}_x64-setup.exe.sig"

# Upload EXE
Write-Host "  Uploading NSIS installer (.exe)..." -ForegroundColor Gray
$ExeUploadUrl = "https://uploads.github.com/repos/$Repo/releases/$ReleaseId/assets?name=Stiftung.der.DPolG.Buchungssystem_${Version}_x64-setup.exe"
$ExeBytes = [System.IO.File]::ReadAllBytes($ExeFile)
Invoke-RestMethod -Uri $ExeUploadUrl -Method Post -Headers $Headers -Body $ExeBytes -ContentType "application/octet-stream" | Out-Null
Write-Host "  Installer uploaded!" -ForegroundColor Green

# Upload SIG
Write-Host "  Uploading signature (.sig)..." -ForegroundColor Gray
$SigUploadUrl = "https://uploads.github.com/repos/$Repo/releases/$ReleaseId/assets?name=Stiftung.der.DPolG.Buchungssystem_${Version}_x64-setup.exe.sig"
$SigBytes = [System.IO.File]::ReadAllBytes($SigFile)
Invoke-RestMethod -Uri $SigUploadUrl -Method Post -Headers $Headers -Body $SigBytes -ContentType "application/octet-stream" | Out-Null
Write-Host "  Signature uploaded!" -ForegroundColor Green

# Create and upload latest.json
Write-Host "  Creating and uploading latest.json..." -ForegroundColor Gray
$Signature = Get-Content $SigFile -Raw
$PubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$LatestJson = @{
    version = $Version
    notes = "Release v$Version"
    pub_date = $PubDate
    platforms = @{
        "windows-x86_64" = @{
            signature = $Signature.Trim()
            url = "https://github.com/$Repo/releases/download/v$Version/Stiftung.der.DPolG.Buchungssystem_${Version}_x64-setup.exe"
        }
    }
} | ConvertTo-Json -Depth 4

$LatestJsonUrl = "https://uploads.github.com/repos/$Repo/releases/$ReleaseId/assets?name=latest.json"
$LatestJsonBytes = [System.Text.Encoding]::UTF8.GetBytes($LatestJson)
Invoke-RestMethod -Uri $LatestJsonUrl -Method Post -Headers $Headers -Body $LatestJsonBytes -ContentType "application/json" | Out-Null
Write-Host "  latest.json uploaded!" -ForegroundColor Green

Write-Host ""
Write-Host "Release v$Version erfolgreich veroeffentlicht!" -ForegroundColor Cyan
Write-Host ""
Write-Host "Release URL: https://github.com/$Repo/releases/tag/v$Version" -ForegroundColor Blue
Write-Host ""
Write-Host "Naechster Schritt: Auto-Update testen" -ForegroundColor Yellow
Write-Host "   1. Oeffnen Sie die installierte App (aeltere Version)"
Write-Host "   2. Update-Dialog sollte erscheinen"
Write-Host "   3. Klicken Sie auf Ja zum Updaten"
Write-Host ""
Write-Host "Quick Release Script completed!" -ForegroundColor Green
