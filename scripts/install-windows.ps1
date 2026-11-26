<#
.SYNOPSIS
    NetFreedom Windows Desktop & Service Installation Script
.DESCRIPTION
    Installs NetFreedom Windows GUI application, background service, drivers,
    and command-line utilities. Supports both Administrator and Standard user installations.
#>

param(
    [string]$InstallPath = "",
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

function Check-Administrator {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    return $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

$IsAdmin = Check-Administrator

if (-not $InstallPath) {
    if ($IsAdmin) {
        $InstallPath = "$env:ProgramFiles\NetFreedom"
    } else {
        $InstallPath = "$env:LOCALAPPDATA\Programs\NetFreedom"
    }
}

if ($Uninstall) {
    Write-Host "[+] Stopping NetFreedom processes and services..."
    if ($IsAdmin) {
        Stop-Service -Name "NetFreedomDaemon" -ErrorAction SilentlyContinue
        sc.exe delete "NetFreedomDaemon" 2>$null | Out-Null
        netsh advfirewall firewall delete rule name="NetFreedom_Block_QUIC" 2>$null | Out-Null
    }
    taskkill /F /IM NetFreedom-GUI.exe /IM ciadpi.exe /IM goodbyedpi.exe /IM sing-box.exe /IM netfreedom.exe /IM netfreedom-daemon.exe 2>$null | Out-Null

    # Reset System Proxy in Registry
    reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f 2>$null | Out-Null
    
    # Flush WinINet proxy cache in memory
    try {
        $sig = '[DllImport("wininet.dll")] public static extern bool InternetSetOption(IntPtr hInternet, int dwOption, IntPtr lpBuffer, int dwBufferLength);'
        $w = Add-Type -MemberDefinition $sig -Name 'WNet' -Namespace 'NetFreedom' -PassThru
        $w::InternetSetOption([IntPtr]::Zero, 39, [IntPtr]::Zero, 0) | Out-Null
        $w::InternetSetOption([IntPtr]::Zero, 37, [IntPtr]::Zero, 0) | Out-Null
    } catch {}

    # Remove Desktop Shortcuts
    $DesktopPaths = @(
        [Environment]::GetFolderPath("CommonDesktopDirectory"),
        [Environment]::GetFolderPath("Desktop")
    )
    foreach ($dp in $DesktopPaths) {
        if (Test-Path "$dp\NetFreedom.lnk") {
            Remove-Item "$dp\NetFreedom.lnk" -Force -ErrorAction SilentlyContinue
        }
    }

    # Remove from System or User PATH
    $TargetEnv = if ($IsAdmin) { "Machine" } else { "User" }
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", $TargetEnv)
    if ($CurrentPath -like "*$InstallPath*") {
        $NewPath = ($CurrentPath.Split(';') | Where-Object { $_ -ne $InstallPath -and $_ -ne "" }) -join ';'
        [Environment]::SetEnvironmentVariable("Path", $NewPath, $TargetEnv)
    }

    # Remove Install Directory
    if (Test-Path $InstallPath) {
        Remove-Item -Path $InstallPath -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "[+] NetFreedom has been completely uninstalled and removed."
    exit 0
}

Write-Host "=== NetFreedom Windows Installer ==="
if ($IsAdmin) {
    Write-Host "[*] Running with Administrative privileges (System-wide install)."
} else {
    Write-Host "[*] Running as Standard User (Per-user install without UAC requirement)."
}

# 1. Create directory
if (-not (Test-Path $InstallPath)) {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# 2. Copy binaries
Write-Host "[+] Copying binaries to $InstallPath..."
$Binaries = @(
    "netfreedom.exe",
    "NetFreedom-GUI.exe",
    "ciadpi.exe",
    "goodbyedpi.exe",
    "WinDivert.dll",
    "WinDivert64.sys",
    "netfreedom-daemon.exe",
    "sing-box.exe",
    "wintun.dll",
    "Uninstall.bat"
)
foreach ($bin in $Binaries) {
    if (Test-Path "$ScriptDir\$bin") {
        Copy-Item "$ScriptDir\$bin" -Destination "$InstallPath\$bin" -Force -ErrorAction SilentlyContinue
    }
}
Get-ChildItem -Path $InstallPath -Recurse | Unblock-File -ErrorAction SilentlyContinue

# 3. Verify Microsoft Edge WebView2 Runtime
Write-Host "[+] Checking Microsoft Edge WebView2 Runtime..."
$WebViewFound = $false
$WebViewKeys = @(
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-F501-47DD-8A04-6876D2EBE2C2}",
    "HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-F501-47DD-8A04-6876D2EBE2C2}"
)
foreach ($key in $WebViewKeys) {
    if (Test-Path $key) {
        $pv = (Get-ItemProperty -Path $key -ErrorAction SilentlyContinue).pv
        if ($pv) {
            $WebViewFound = $true
            break
        }
    }
}
if (-not $WebViewFound) {
    Write-Host "[!] Microsoft Edge WebView2 not detected. Downloading bootstrapper..."
    $WvInstaller = "$env:TEMP\MicrosoftEdgeWebview2Setup.exe"
    Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $WvInstaller
    Start-Process $WvInstaller -ArgumentList '/silent /install' -Wait
    Write-Host "[+] Microsoft Edge WebView2 provisioned."
}

# 4. Add to PATH
$TargetEnv = if ($IsAdmin) { "Machine" } else { "User" }
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", $TargetEnv)
if ($CurrentPath -notlike "*$InstallPath*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallPath", $TargetEnv)
    Write-Host "[+] Added $InstallPath to $TargetEnv PATH."
}

# 5. Create Desktop Shortcut for NetFreedom-GUI
try {
    $WshShell = New-Object -ComObject WScript.Shell
    $DesktopPath = if ($IsAdmin) { [Environment]::GetFolderPath("CommonDesktopDirectory") } else { [Environment]::GetFolderPath("Desktop") }
    $Shortcut = $WshShell.CreateShortcut("$DesktopPath\NetFreedom.lnk")
    $Shortcut.TargetPath = "$InstallPath\NetFreedom-GUI.exe"
    $Shortcut.WorkingDirectory = "$InstallPath"
    $Shortcut.Description = "NetFreedom Universal Censorship Circumvention Suite"
    $Shortcut.Save()

    if ($IsAdmin) {
        # Mark shortcut as Run as Administrator if elevated
        $lnkPath = "$DesktopPath\NetFreedom.lnk"
        if (Test-Path $lnkPath) {
            $bytes = [System.IO.File]::ReadAllBytes($lnkPath)
            $bytes[21] = $bytes[21] -bor 0x20
            [System.IO.File]::WriteAllBytes($lnkPath, $bytes)
        }
    }
    Write-Host "[+] Created Desktop shortcut: NetFreedom"
} catch {
    Write-Warning "Could not create desktop shortcut: $_"
}

# 6. Create Windows Service (Only if Administrator)
if ($IsAdmin -and (Test-Path "$InstallPath\netfreedom-daemon.exe")) {
    Write-Host "[+] Registering Windows Service 'NetFreedomDaemon'..."
    sc.exe create "NetFreedomDaemon" binPath= "`"$InstallPath\netfreedom-daemon.exe`"" start= auto DisplayName= "NetFreedom Network Service" 2>$null | Out-Null
    sc.exe description "NetFreedomDaemon" "NetFreedom Multi-Mode Privacy and Network Engine" 2>$null | Out-Null
    Start-Service -Name "NetFreedomDaemon" -ErrorAction SilentlyContinue
}

Write-Host "=== NetFreedom Windows Installation Complete! ==="
Write-Host "You can now run 'NetFreedom' from your Desktop or 'netfreedom' in PowerShell/CMD."
