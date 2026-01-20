# bver installer for Windows
# Usage: powershell -ExecutionPolicy ByPass -c "irm https://github.com/flaport/bver/releases/latest/download/install.ps1 | iex"

$ErrorActionPreference = "Stop"

$Repo = "flaport/bver"
$BinaryName = "bver"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

function Get-LatestVersion {
    $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $response.tag_name
}

function Main {
    $Target = "x86_64-pc-windows-msvc"
    $Version = Get-LatestVersion

    if (-not $Version) {
        Write-Error "Could not determine latest version"
        exit 1
    }

    Write-Host "Installing bver $Version for $Target..."

    # Create install directory if it doesn't exist
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    }

    # Download
    $DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$BinaryName-$Target.zip"
    $TempFile = Join-Path $env:TEMP "bver-$Version.zip"

    Write-Host "Downloading from $DownloadUrl..."
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile

    # Extract
    Expand-Archive -Path $TempFile -DestinationPath $InstallDir -Force
    Remove-Item $TempFile

    Write-Host ""
    Write-Host "Successfully installed bver to $InstallDir\$BinaryName.exe"
    Write-Host ""

    # Check if install dir is in PATH
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        Write-Host "Add $InstallDir to your PATH to use bver:"
        Write-Host ""
        Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$InstallDir', 'User')"
        Write-Host ""
        Write-Host "Or add it manually via System Properties > Environment Variables."
    } else {
        Write-Host "Run 'bver --help' to get started."
    }
}

Main
