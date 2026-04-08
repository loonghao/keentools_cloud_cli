<#
.SYNOPSIS
    Install keentools-cloud CLI on Windows.

.DESCRIPTION
    Downloads the latest (or a pinned) release binary of keentools-cloud from
    GitHub and installs it to a local directory.

    Compatible with both direct execution and `irm ... | iex` piped invocation.

.EXAMPLE
    irm https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.ps1 | iex

.EXAMPLE
    $env:KEENTOOLS_INSTALL_VERSION = "v0.2.0"
    irm https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.ps1 | iex

.EXAMPLE
    .\install.ps1
#>

$ErrorActionPreference = "Stop"

# ---------- resolve parameters from environment variables ----------------------
# NOTE: param() blocks are not supported with `irm ... | iex` piping.
# Use environment variables to customise the install:
#   $env:KEENTOOLS_INSTALL_VERSION    = "v0.2.0"   (default: latest)
#   $env:KEENTOOLS_INSTALL_DIR        = "C:\my\bin" (default: ~\.keentools\bin)
#   $env:KEENTOOLS_INSTALL_REPOSITORY = "owner/repo"

$Version    = if ($env:KEENTOOLS_INSTALL_VERSION)    { $env:KEENTOOLS_INSTALL_VERSION }    else { "latest" }
$InstallDir = if ($env:KEENTOOLS_INSTALL_DIR)        { $env:KEENTOOLS_INSTALL_DIR }        else { "$env:USERPROFILE\.keentools\bin" }
$Repository = if ($env:KEENTOOLS_INSTALL_REPOSITORY) { $env:KEENTOOLS_INSTALL_REPOSITORY } else { "loonghao/keentools_cloud_cli" }

# ---------- detect architecture -----------------------------------------------
# Prefer RuntimeInformation (works on PowerShell 7 / .NET 4.7.1+).
# Fall back to $env:PROCESSOR_ARCHITECTURE for older Windows PowerShell 5.1.

$arch = $null
try {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
} catch {
    # RuntimeInformation unavailable (older .NET Framework) — use env var
}

if (-not $arch) {
    $pa = $env:PROCESSOR_ARCHITECTURE
    switch ($pa) {
        "AMD64" { $target = "x86_64-pc-windows-msvc" }
        "ARM64" { $target = "aarch64-pc-windows-msvc" }
        default { throw "Unsupported architecture: PROCESSOR_ARCHITECTURE=$pa" }
    }
} else {
    switch ($arch.ToString()) {
        "X64"   { $target = "x86_64-pc-windows-msvc" }
        "Arm64" { $target = "aarch64-pc-windows-msvc" }
        default { throw "Unsupported architecture: $arch" }
    }
}

# ---------- resolve download URL ----------------------------------------------

if ($Version -eq "latest") {
    $url = "https://github.com/$Repository/releases/latest/download/keentools-cloud-$target.zip"
} else {
    if (-not $Version.StartsWith("v")) { $Version = "v$Version" }
    $url = "https://github.com/$Repository/releases/download/$Version/keentools-cloud-$target.zip"
}

Write-Host "-> Downloading keentools-cloud ($Version) for $target..."
Write-Host "   $url"

# ---------- download & extract ------------------------------------------------

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
    $zip = Join-Path $tmp "keentools-cloud.zip"
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing

    Expand-Archive -Path $zip -DestinationPath $tmp -Force

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -Path (Join-Path $tmp "keentools-cloud.exe") -Destination (Join-Path $InstallDir "keentools-cloud.exe") -Force

    Write-Host "v Installed keentools-cloud to $InstallDir\keentools-cloud.exe"
} finally {
    Remove-Item -Recurse -Force -Path $tmp -ErrorAction SilentlyContinue
}

# ---------- PATH hint ---------------------------------------------------------

$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($currentPath -notlike "*$InstallDir*") {
    Write-Host ""
    Write-Host "!  $InstallDir is not in your PATH."
    Write-Host "   Add it with:"
    Write-Host ""
    Write-Host "   `$env:PATH = `"$InstallDir;`$env:PATH`""
    Write-Host ""
    Write-Host "   Or permanently (requires restart):"
    Write-Host "   [Environment]::SetEnvironmentVariable('PATH', `"$InstallDir;`$currentPath`", 'User')"
    Write-Host ""
}

Write-Host ""
Write-Host "Get started:"
Write-Host "   `$env:KEENTOOLS_API_URL   = 'https://your-api-endpoint.example.com'"
Write-Host "   `$env:KEENTOOLS_API_TOKEN = 'your-token-here'"
Write-Host "   keentools-cloud --help"
