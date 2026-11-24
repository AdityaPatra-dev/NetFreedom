<#
.SYNOPSIS
    NetFreedom Windows Desktop & Service Installation Script
.DESCRIPTION
    Installs NetFreedom Windows GUI application, background service, drivers,
    and command-line utilities.
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
    Write-Host "[+] Stopping NetFreedom processes and services..."
    Stop-Service -Name "NetFreedomDaemon" -ErrorAction SilentlyContinue
    sc.exe delete "NetFreedomDaemon" | Out-Null
    taskkill /F /IM NetFreedom-GUI.exe /IM ciadpi.exe /IM goodbyedpi.exe /IM sing-box.exe 2>$null | Out-Null

    # Reset System Proxy in Registry
    reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f | Out-Null
    netsh advfirewall firewall delete rule name="NetFreedom_Block_QUIC" 2>$null | Out-Null

    # Remove Desktop Shortcut
    $DesktopPath = [Environment]::GetFolderPath("CommonDesktopDirectory")
    if (Test-Path "$DesktopPath\NetFreedom.lnk") {
        Remove-Item "$DesktopPath\NetFreedom.lnk" -Force
    }

    # Remove from System PATH
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    if ($CurrentPath -like "*$InstallPath*") {
        $NewPath = ($CurrentPath.Split(';') | Where-Object { $_ -ne $InstallPath -and $_ -ne "" }) -join ';'
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "Machine")
    }

    # Remove Install Directory
    if (Test-Path $InstallPath) {
        Remove-Item -Path $InstallPath -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "[+] NetFreedom has been completely uninstalled and removed."
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
Copy-Item "$ScriptDir\NetFreedom-GUI.exe" -Destination "$InstallPath\NetFreedom-GUI.exe" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\ciadpi.exe" -Destination "$InstallPath\ciadpi.exe" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\goodbyedpi.exe" -Destination "$InstallPath\goodbyedpi.exe" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\WinDivert.dll" -Destination "$InstallPath\WinDivert.dll" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\WinDivert64.sys" -Destination "$InstallPath\WinDivert64.sys" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\netfreedom-daemon.exe" -Destination "$InstallPath\netfreedom-daemon.exe" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\sing-box.exe" -Destination "$InstallPath\sing-box.exe" -Force -ErrorAction SilentlyContinue
Copy-Item "$ScriptDir\wintun.dll" -Destination "$InstallPath\wintun.dll" -Force -ErrorAction SilentlyContinue
Get-ChildItem -Path $InstallPath -Recurse | Unblock-File -ErrorAction SilentlyContinue

# 3. Add to system PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($CurrentPath -notlike "*$InstallPath*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallPath", "Machine")
    Write-Host "[+] Added $InstallPath to system PATH."
}

# 4. Create Desktop Shortcut for NetFreedom-GUI
try {
    $WshShell = New-Object -ComObject WScript.Shell
    $DesktopPath = [Environment]::GetFolderPath("CommonDesktopDirectory")
    $Shortcut = $WshShell.CreateShortcut("$DesktopPath\NetFreedom.lnk")
    $Shortcut.TargetPath = "$InstallPath\NetFreedom-GUI.exe"
    $Shortcut.WorkingDirectory = "$InstallPath"
    $Shortcut.Description = "NetFreedom Desktop GUI"
    $Shortcut.Save()
    # Set Run as Administrator byte on shortcut (offset 21 |= 0x20)
    $lnkPath = "$DesktopPath\NetFreedom.lnk"
    if (Test-Path $lnkPath) {
        $bytes = [System.IO.File]::ReadAllBytes($lnkPath)
        $bytes[21] = $bytes[21] -bor 0x20
        [System.IO.File]::WriteAllBytes($lnkPath, $bytes)
    }
    Write-Host "[+] Created Elevated Desktop shortcut: NetFreedom"
} catch {
    Write-Warning "Could not create desktop shortcut: $_"
}

# 5. Create Windows Service
Write-Host "[+] Registering Windows Service 'NetFreedomDaemon'..."
sc.exe create "NetFreedomDaemon" binPath= "`"$InstallPath\netfreedom-daemon.exe`"" start= auto DisplayName= "NetFreedom Network Service" | Out-Null
sc.exe description "NetFreedomDaemon" "NetFreedom Multi-Mode Privacy and Network Engine" | Out-Null

# 6. Start service
Start-Service -Name "NetFreedomDaemon" -ErrorAction SilentlyContinue

Write-Host "=== NetFreedom Windows Installation Complete! ==="
Write-Host "You can now run 'NetFreedom' from your Desktop or 'netfreedom' in any terminal."
