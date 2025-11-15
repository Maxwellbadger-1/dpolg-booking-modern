# Upload latest.json to GitHub Release
$ErrorActionPreference = "Stop"

# Read GitHub token
$token = (Get-Content .github-token -Raw).Trim()
$env:GITHUB_TOKEN = $token

# Set headers
$headers = @{
    'Authorization' = "token $token"
    'Accept' = 'application/vnd.github.v3+json'
}

# Upload URL (from previous release creation)
$uploadUrl = 'https://uploads.github.com/repos/Maxwellbadger-1/dpolg-booking-modern/releases/257758166/assets'

Write-Host "Uploading latest.json..."

# Read latest.json as bytes
$latestJsonContent = [System.IO.File]::ReadAllBytes('latest.json')

# Upload
$result = Invoke-RestMethod -Uri "$uploadUrl`?name=latest.json" -Method Post -Headers $headers -Body $latestJsonContent -ContentType 'application/json'

Write-Host "✅ latest.json uploaded successfully!"
Write-Host "Name: $($result.name)"
Write-Host "Size: $($result.size) bytes"
Write-Host "State: $($result.state)"
Write-Host "Download URL: $($result.browser_download_url)"
