# Universal installer for terminal — GPU-rendered terminal emulator with GTD.
# Usage: irm https://raw.githubusercontent.com/CySpiegel/terminal/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "CySpiegel/terminal"
$BinaryName = "terminal"
$GitHubApi = "https://api.github.com/repos/$Repo/releases/latest"

function Write-Info($msg)  { Write-Host "info  " -ForegroundColor Cyan -NoNewline; Write-Host $msg }
function Write-Ok($msg)    { Write-Host "ok    " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Warn($msg)  { Write-Host "warn  " -ForegroundColor Yellow -NoNewline; Write-Host $msg }
function Write-Err($msg)   { Write-Host "error " -ForegroundColor Red -NoNewline; Write-Host $msg; exit 1 }

# Detect architecture
function Get-Arch {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64"   { return "x86_64" }
        "x86"     { return "x86_64" }  # Fallback
        "ARM64"   { return "aarch64" }
        default   { Write-Err "Unsupported architecture: $arch" }
    }
}

# Resolve version
function Get-Version {
    if ($env:TERMINAL_VERSION) {
        return $env:TERMINAL_VERSION
    }

    Write-Info "Fetching latest release..."
    try {
        $release = Invoke-RestMethod -Uri $GitHubApi -Headers @{ "User-Agent" = "terminal-installer" }
        return $release.tag_name
    } catch {
        Write-Err "Failed to fetch latest release: $_"
    }
}

# Main
function Install-Terminal {
    Write-Info "Installing $BinaryName..."

    $Arch = Get-Arch
    $Version = Get-Version
    $Target = "$Arch-pc-windows-msvc"

    Write-Info "Platform: $Target"
    Write-Info "Version:  $Version"

    # Install directory
    $InstallDir = if ($env:TERMINAL_INSTALL_DIR) {
        $env:TERMINAL_INSTALL_DIR
    } else {
        "$env:USERPROFILE\.terminal\bin"
    }

    # Create temp directory
    $TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "terminal-install-$(Get-Random)"
    New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null

    try {
        # Download
        $Archive = "$BinaryName-$Version-$Target.zip"
        $ArchiveUrl = "https://github.com/$Repo/releases/download/$Version/$Archive"
        $ChecksumsUrl = "https://github.com/$Repo/releases/download/$Version/checksums-sha256.txt"
        $ArchivePath = Join-Path $TmpDir $Archive
        $ChecksumsPath = Join-Path $TmpDir "checksums-sha256.txt"

        Write-Info "Downloading $Archive..."
        Invoke-WebRequest -Uri $ArchiveUrl -OutFile $ArchivePath -UseBasicParsing

        Write-Info "Downloading checksums..."
        try {
            Invoke-WebRequest -Uri $ChecksumsUrl -OutFile $ChecksumsPath -UseBasicParsing
        } catch {
            Write-Warn "Could not download checksums"
        }

        # Verify checksum
        if (Test-Path $ChecksumsPath) {
            $expectedLine = Get-Content $ChecksumsPath | Where-Object { $_ -match $Archive }
            if ($expectedLine) {
                $expected = ($expectedLine -split '\s+')[0]
                $actual = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash.ToLower()
                if ($expected -ne $actual) {
                    Write-Err "Checksum mismatch! Expected: $expected, Got: $actual"
                }
                Write-Ok "Checksum verified"
            }
        }

        # Extract
        Write-Info "Extracting..."
        Expand-Archive -Path $ArchivePath -DestinationPath $TmpDir -Force

        # Install
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Copy-Item -Path (Join-Path $TmpDir "$BinaryName.exe") -Destination (Join-Path $InstallDir "$BinaryName.exe") -Force

        Write-Ok "Installed $BinaryName $Version to $InstallDir\$BinaryName.exe"

        # Add to PATH
        $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($UserPath -notlike "*$InstallDir*") {
            [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
            $env:Path = "$env:Path;$InstallDir"
            Write-Ok "Added $InstallDir to user PATH"
        }

    } finally {
        # Cleanup
        Remove-Item -Path $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Install-Terminal
