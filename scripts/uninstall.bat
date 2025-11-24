@echo off
setlocal enabledelayedexpansion

echo Uninstalling Discord RPC plugin for DeaDBeeF...

set "PLUGIN_DIR=%APPDATA%\deadbeef"
set "EXTENSION=dll"
set "PLUGIN_FILE=%PLUGIN_DIR%\discordrpc.%EXTENSION%"

:: Check if plugin exists
if not exist "%PLUGIN_FILE%" (
    echo Plugin not found at: %PLUGIN_FILE%
    echo Nothing to uninstall.
    exit /b 0
)

:: Remove the plugin
echo Removing plugin: %PLUGIN_FILE%
del /F /Q "%PLUGIN_FILE%"

if %ERRORLEVEL% neq 0 (
    echo Failed to remove plugin.
    exit /b %ERRORLEVEL%
)

echo.
echo Uninstallation complete!
echo Plugin removed from: %PLUGIN_FILE%
echo.
echo Please restart DeaDBeeF for changes to take effect.

endlocal
