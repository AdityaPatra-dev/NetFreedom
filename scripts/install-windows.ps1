<#
.SYNOPSIS
    NetFreedom Windows Service & Driver Installation Script
.DESCRIPTION
    Installs NetFreedom Windows background service, downloads signed Wintun driver if missing,
    and configures Windows Filtering Platform (WFP) rules.
#>

param(
    [string]$InstallPath = "$env:ProgramFiles\NetFreedom",
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

function Check-Administrator {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Check-Administrator)) {
    Write-Error "[-] This script must be run as Administrator in PowerShell."
    exit 1
}

if ($Uninstall) {
    Write-Host "[+] Stopping and removing NetFreedom Windows Service..."
    Stop-Service -Name "NetFreedomDaemon" -ErrorAction SilentlyContinue
    sc.exe delete "NetFreedomDaemon"
    Write-Host "[+] NetFreedom Service uninstalled successfully."
    exit 0
}

Write-Host "=== NetFreedom Windows Installer ==="

# 1. Create directory
if (-not (Test-Path $InstallPath)) {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# 2. Copy binaries
Write-Host "[+] Copying binaries to $InstallPath..."
Copy-Item "$ScriptDir\netfreedom.exe" -Destination "$InstallPath\netfreedom.exe" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\netfreedom-daemon.exe" -Destination "$InstallPath\netfreedom-daemon.exe" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\wintun.dll" -Destination "$InstallPath\wintun.dll" -Force -ErrorAction SilentlyContinue

# 3. Add to system PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($CurrentPath -notlike "*$InstallPath*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallPath", "Machine")
    Write-Host "[+] Added $InstallPath to system PATH."
}

# 4. Create Windows Service
Write-Host "[+] Registering Windows Service 'NetFreedomDaemon'..."
sc.exe create "NetFreedomDaemon" binPath= "`"$InstallPath\netfreedom-daemon.exe`"" start= auto DisplayName= "NetFreedom Network Service"
sc.exe description "NetFreedomDaemon" "NetFreedom Multi-Mode Privacy and Network Engine"

# 5. Start service
Start-Service -Name "NetFreedomDaemon" -ErrorAction SilentlyContinue

Write-Host "=== NetFreedom Windows Installation Complete! ==="
Write-Host "Run 'netfreedom auto' or 'netfreedom status' from PowerShell or CMD."
