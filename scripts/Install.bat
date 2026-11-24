@echo off
title NetFreedom Installer
echo ==============================================
echo       NetFreedom 1-Click Desktop Installer
echo ==============================================
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\Programs\NetFreedom"
set "SCRIPT_DIR=%~dp0"

echo [*] Installing NetFreedom to: %INSTALL_DIR%
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo [*] Copying application files...
copy /Y "%SCRIPT_DIR%NetFreedom-GUI.exe" "%INSTALL_DIR%\" >nul
copy /Y "%SCRIPT_DIR%ciadpi.exe" "%INSTALL_DIR%\" >nul
copy /Y "%SCRIPT_DIR%netfreedom.exe" "%INSTALL_DIR%\" >nul 2>nul
copy /Y "%SCRIPT_DIR%wintun.dll" "%INSTALL_DIR%\" >nul 2>nul
copy /Y "%SCRIPT_DIR%Uninstall.bat" "%INSTALL_DIR%\" >nul 2>nul

echo [*] Creating Desktop Shortcut...
powershell -NoProfile -Command "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut([Environment]::GetFolderPath('Desktop') + '\NetFreedom.lnk'); $s.TargetPath = '%INSTALL_DIR%\NetFreedom-GUI.exe'; $s.WorkingDirectory = '%INSTALL_DIR%'; $s.Description = 'NetFreedom Universal Bypass'; $s.Save()"

echo.
echo ==============================================
echo  [+] Installation Successful!
echo  [+] A "NetFreedom" shortcut is now on your Desktop.
echo.
echo  You can now simply double-click "NetFreedom"
echo  on your Desktop anytime you want to connect.
echo ==============================================
echo.
pause
