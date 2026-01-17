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

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

Write-Host "Extracting to $InstallDir..."
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force

$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host "Adding to system PATH..."
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$CurrentPath;$InstallDir",
        "Machine"
    )
}

Write-Host "Checking API $Endpoint..."

$EndpointEnv = [Environment]::GetEnvironmentVariable("LUCKYDRIVE_API_ENDPOINT", "Machine")
if ($EndpointEnv -notlike "*$Endpoint*") {
    Write-Host "Adding API env..."
    [Environment]::SetEnvironmentVariable(
        "LUCKYDRIVE_API_ENDPOINT",
        "$Endpoint",
        "Machine"
        )
}


$IconUrl = "https://raw.githubusercontent.com/Lucky2307/luckydrive-cli-rust/refs/heads/master/assets/icon.ico"
$IconPath = "$InstallDir\icon.ico"
Invoke-WebRequest -Uri $IconUrl -OutFile $IconPath -UseBasicParsing

Write-Host "Adding to registry..."

$KeyPath = "HKCU:\SOFTWARE\Classes\SystemFileAssociations\.mp4\shell\UploadToLuckyDrive"
if (-not (Test-Path $KeyPath)) {
    Write-Host "Creating key..."
    New-Item -Path $KeyPath -Force | Out-Null
}
Set-ItemProperty -Path $KeyPath -Name "(Default)" -Value "Upload to LuckyDrive"
Set-ItemProperty -Path $KeyPath -Name "Icon" -Value $IconPath

$CommandKey = "$KeyPath\command"
if (-not (Test-Path $CommandKey)) {
    New-Item -Path $CommandKey -Force | Out-Null
}
Set-ItemProperty -Path $commandKey -Name "(Default)" -Value 'cmd.exe /k luckydrive-cli upload "%1"'

Remove-Item $TempDir -Recurse -Force

Write-Host "Installed successfully."
