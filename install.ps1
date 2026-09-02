param(
  [ValidateSet("install", "uninstall")]
  [string]$Action = "install",
  [string]$RepoUrl = "https://github.com/xscriptor/gitnapse.git",
  [switch]$Cleanup
)

$ErrorActionPreference = "Stop"

function Log([string]$Message) {
  Write-Host "[gitnapse-install] $Message"
}

function Ensure-CargoPath {
  $cargoBin = Join-Path $HOME ".cargo\bin"
  if (Test-Path $cargoBin) {
    if (-not ($env:PATH.Split(";") -contains $cargoBin)) {
      $env:PATH = "$cargoBin;$env:PATH"
    }
  }
}

function Ensure-Rust {
  Ensure-CargoPath
  if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Log "Rust toolchain already installed."
    return
  }

  Log "Rust not found. Installing rustup..."
  if (Get-Command winget -ErrorAction SilentlyContinue) {
    winget install --id Rustlang.Rustup -e --accept-package-agreements --accept-source-agreements
  } else {
    $tmp = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $tmp
    & $tmp -y
    Remove-Item $tmp -Force -ErrorAction SilentlyContinue
  }

  Ensure-CargoPath
  if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found after rustup installation. Restart shell and retry."
  }
}

function Install-GitNapse {
  Ensure-Rust
  Log "Installing gitnapse from $RepoUrl ..."
  cargo install --git $RepoUrl --locked gitnapse
  Log "Installed. Run: gitnapse --help"
}

function Uninstall-GitNapse {
  Ensure-CargoPath
  if (Get-Command cargo -ErrorAction SilentlyContinue) {
    try {
      cargo uninstall gitnapse | Out-Null
      Log "Uninstalled gitnapse."
    } catch {
      Log "gitnapse is not currently installed via cargo."
    }
  } else {
    Log "cargo not found; skipping cargo uninstall."
  }

  if ($Cleanup.IsPresent) {
    Remove-Item -Recurse -Force "$env:APPDATA\GitNapse" -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force "$env:LOCALAPPDATA\GitNapse" -ErrorAction SilentlyContinue
    Log "Removed local config/cache directories."
  }
}

if ($Action -eq "install") {
  Install-GitNapse
} else {
  Uninstall-GitNapse
}
