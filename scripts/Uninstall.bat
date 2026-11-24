@echo off
title NetFreedom Uninstaller
echo ==============================================
echo       NetFreedom 1-Click Uninstaller
echo ==============================================
echo.

echo [*] Closing any active NetFreedom processes...
taskkill /F /IM NetFreedom-GUI.exe /IM ciadpi.exe /IM goodbyedpi.exe /IM netfreedom.exe /IM netfreedom-daemon.exe 2>nul

echo [*] Restoring direct network connection (disabling proxy)...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f >nul
netsh advfirewall firewall delete rule name="NetFreedom_Block_QUIC" >nul 2>nul
powershell -NoProfile -Command "$sig = '[DllImport(\"wininet.dll\")] public static extern bool InternetSetOption(IntPtr hInternet, int dwOption, IntPtr lpBuffer, int dwBufferLength);'; $w = Add-Type -MemberDefinition $sig -Name 'W' -Namespace 'N' -PassThru; $w::InternetSetOption([IntPtr]::Zero, 39, [IntPtr]::Zero, 0); $w::InternetSetOption([IntPtr]::Zero, 37, [IntPtr]::Zero, 0)" 2>nul

echo [*] Removing Desktop shortcut...
del /F /Q "%USERPROFILE%\Desktop\NetFreedom.lnk" 2>nul

echo [*] Removing installed files...
set "INSTALL_DIR=%LOCALAPPDATA%\Programs\NetFreedom"
if exist "%INSTALL_DIR%" (
    rmdir /S /Q "%INSTALL_DIR%"
)

echo.
echo ==============================================
echo  [+] NetFreedom has been completely removed!
echo  [+] Normal internet settings have been restored.
echo ==============================================
echo.
pause
