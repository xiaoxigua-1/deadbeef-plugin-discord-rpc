@echo off
setlocal enabledelayedexpansion

echo Building Discord RPC plugin for DeaDBeeF...
cargo build --release

if %ERRORLEVEL% neq 0 (
    echo Build failed!
    exit /b %ERRORLEVEL%
)

set "PLUGIN_DIR=%APPDATA%\deadbeef"
set "EXTENSION=dll"

:: Find the compiled .dll file
set "PLUGIN_FILE="
for %%f in (target\release\*.dll) do (
    set "PLUGIN_FILE=%%f"
    goto :found
)

:found
if not defined PLUGIN_FILE (
    echo Error: Could not find compiled plugin file in target\release\
    exit /b 1
)

echo Found plugin file: %PLUGIN_FILE%

:: Create plugin directory if it doesn't exist
if not exist "%PLUGIN_DIR%" (
    echo Creating plugin directory: %PLUGIN_DIR%
    mkdir "%PLUGIN_DIR%"
)

:: Copy and rename the plugin
set "TARGET_FILE=%PLUGIN_DIR%\discordrpc.%EXTENSION%"
echo Installing plugin to: %TARGET_FILE%
copy /Y "%PLUGIN_FILE%" "%TARGET_FILE%"

if %ERRORLEVEL% neq 0 (
    echo Installation failed!
    exit /b %ERRORLEVEL%
)

echo.
echo Installation complete!
echo Plugin installed to: %TARGET_FILE%
echo.
echo Please restart DeaDBeeF to load the plugin.

endlocal
