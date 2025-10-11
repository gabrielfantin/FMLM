# ============================
# PowerShell bootstrap for FMLM + FFmpeg + LLVM + MSVC(x64)
# ============================

$ErrorActionPreference = "Stop"

# --- Helpers ---
function Add-ToPath($p) {
  if ([string]::IsNullOrWhiteSpace($p)) { return }
  $parts = $env:PATH -split ';' | Where-Object { $_ -ne '' }
  if (-not ($parts | Where-Object { $_.TrimEnd('\') -ieq $p.TrimEnd('\') })) {
    $env:PATH = ($parts + $p) -join ';'
  }
}

function Prepend-Env([string]$name, [string]$val) {
  if ([string]::IsNullOrWhiteSpace($val)) { return }
  $current = [Environment]::GetEnvironmentVariable($name, 'Process')
  if ([string]::IsNullOrWhiteSpace($current)) {
    [Environment]::SetEnvironmentVariable($name, $val, 'Process')
  } else {
    $items = $current -split ';'
    if (-not ($items | Where-Object { $_.TrimEnd('\') -ieq $val.TrimEnd('\') })) {
      [Environment]::SetEnvironmentVariable($name, "$val;$current", 'Process')
    }
  }
}

# --- Paths ---
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ProjectRoot

# --- 1) Import MSVC x64 toolchain into this PS process ---
# Prefer VsDevCmd if available (more robust), else fallback to vcvarsall
$vswhere = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
$VsDevCmd = $null
if (Test-Path $vswhere) {
  $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
  if ($vsPath) {
    $tryDev = Join-Path $vsPath 'Common7\Tools\VsDevCmd.bat'
    if (Test-Path $tryDev) { $VsDevCmd = $tryDev }
  }
}
if (-not $VsDevCmd) {
  $VsDevCmd = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat'
}

if (Test-Path $VsDevCmd) {
  # Import env vars from VsDevCmd (x64 host & target)
  $envDump = & cmd.exe /c "`"$VsDevCmd`" -arch=x64 -host_arch=x64 && set"
} else {
  # Fallback: vcvarsall
  $vcvarsall = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat'
  if (-not (Test-Path $vcvarsall)) {
    throw "Could not find VS toolchain. Install 'Visual Studio Build Tools 2022' with 'Desktop development with C++'."
  }
  $envDump = & cmd.exe /c "`"$vcvarsall`" amd64 && set"
}

foreach ($line in $envDump) {
  if ($line -match '^(.*?)=(.*)$') {
    $name = $matches[1]
    $value = $matches[2]
    [Environment]::SetEnvironmentVariable($name, $value, 'Process')
  }
}

# Verify cl.exe is now on PATH
if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
  throw "MSVC cl.exe not found on PATH even after VS env import."
}

# --- 2) FFmpeg env (your local bundle under src-tauri\ffmpeg) ---
$env:FFMPEG_DIR = Join-Path $ProjectRoot 'src-tauri\ffmpeg'
$env:FFMPEG_LIB_DIR = Join-Path $env:FFMPEG_DIR 'lib'
$env:FFMPEG_LIBS_DIR = Join-Path $env:FFMPEG_DIR 'lib'
$env:FFMPEG_INCLUDE_DIR = Join-Path $env:FFMPEG_DIR 'include'
$env:FFMPEG_DLL_PATH = Join-Path $env:FFMPEG_DIR 'bin'
$env:PKG_CONFIG_PATH = Join-Path $env:FFMPEG_LIB_DIR 'pkgconfig'

Prepend-Env LIB $env:FFMPEG_LIB_DIR
Prepend-Env INCLUDE $env:FFMPEG_INCLUDE_DIR
Add-ToPath (Join-Path $env:FFMPEG_DIR 'bin')

Write-Host "[i] FFMPEG_DIR         = $($env:FFMPEG_DIR)"
Write-Host "[i] FFMPEG_LIB_DIR     = $($env:FFMPEG_LIB_DIR)"
Write-Host "[i] FFMPEG_LIBS_DIR    = $($env:FFMPEG_LIBS_DIR)"
Write-Host "[i] FFMPEG_INCLUDE_DIR = $($env:FFMPEG_INCLUDE_DIR)"
Write-Host "[i] FFMPEG_DLL_PATH    = $($env:FFMPEG_DLL_PATH)"
Write-Host "[i] PKG_CONFIG_PATH    = $($env:PKG_CONFIG_PATH)"

# --- 3) LLVM / libclang for bindgen ---
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
if (-not (Test-Path (Join-Path $env:LIBCLANG_PATH 'libclang.dll'))) {
  throw "libclang.dll not found at '$($env:LIBCLANG_PATH)'. Ensure LLVM (64-bit) is installed to C:\Program Files\LLVM."
}
Add-ToPath $env:LIBCLANG_PATH
Write-Host "[i] LIBCLANG_PATH      = $($env:LIBCLANG_PATH)"

# --- 4) Launch Tauri (no recursion into yarn script) ---
$localTauri = Join-Path $ProjectRoot 'node_modules\.bin\tauri.cmd'
if (Test-Path $localTauri) {
  if ($args.Count -gt 0 -and $args[0] -ieq 'build') {
    & $localTauri build
  } else {
    & $localTauri dev
  }
} else {
  if ($args.Count -gt 0 -and $args[0] -ieq 'build') {
    yarn tauri build
  } else {
    yarn tauri dev
  }
}
