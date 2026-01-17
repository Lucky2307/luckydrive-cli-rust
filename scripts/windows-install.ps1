param (
    [string]$Endpoint
)

$InstallDir = "C:\Program Files\LuckyDrive"
$TempDir = Join-Path $env:TEMP "LuckyDrive"
$ApiUrl = "https://api.github.com/repos/Lucky2307/luckydrive-cli-rust/releases/latest"
$AssetPattern = "Windows-msvc-x86_64.zip"

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

Write-Host "Fetching latest release info..."
$Release = Invoke-RestMethod -Uri $ApiUrl -Headers @{
    "User-Agent" = "PowerShell"
}

$ZipAsset = $Release.assets | Where-Object {
    $_.name -like "*$AssetPattern"
} | Select-Object -First 1

if (-not $ZipAsset) {
    throw "ZIP asset matching '$AssetPattern' not found in latest release"
}

$ChecksumAsset = $Release.assets | Where-Object {
    $_.name -eq "$($ZipAsset.name).sha256"
} | Select-Object -First 1

if (-not $ChecksumAsset) {
    throw "Checksum file '$($ZipAsset.name).sha256' not found"
}

$ZipPath = Join-Path $TempDir $ZipAsset.name
$ChecksumPath = "$ZipPath.sha256"

Write-Host "Downloading $($ZipAsset.name)..."
Invoke-WebRequest $ZipAsset.browser_download_url -OutFile $ZipPath

Write-Host "Downloading checksum..."
Invoke-WebRequest $ChecksumAsset.browser_download_url -OutFile $ChecksumPath

Write-Host "Verifying checksum..."
$ExpectedHash = (Get-Content $ChecksumPath).Split(" ")[0]
$ActualHash = (Get-FileHash $ZipPath -Algorithm SHA256).Hash.ToLower()

if ($ExpectedHash.ToLower() -ne $ActualHash) {
    throw "Checksum verification FAILED"
}

Write-Host "Checksum OK"

# Install
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

Write-Host "Extracting to $InstallDir..."
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force

# Add to SYSTEM PATH (no duplicates)
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host "Adding to system PATH..."
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$CurrentPath;$InstallDir",
        "Machine"
    )
}

[System.Environment]::SetEnvironmentVariable("LUCKYDRIVE_API_ENDPOINT", "$Endpoint", 'Machine')


Remove-Item $TempDir -Recurse -Force

Write-Host "Installed successfully."
