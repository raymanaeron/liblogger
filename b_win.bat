@echo off
setlocal

set BUILD_TYPE=debug

if "%1"=="--release" (
    set BUILD_TYPE=release
    echo Building in RELEASE mode
) else (
    echo Building in DEBUG mode
)

echo Building Rusty Logger...
cargo build %1

if %ERRORLEVEL% neq 0 (
    echo Build failed with error code %ERRORLEVEL%
    exit /b %ERRORLEVEL%
)

echo Copying config file to target\%BUILD_TYPE% directory...
copy app_config.toml target\%BUILD_TYPE%\

if %ERRORLEVEL% neq 0 (
    echo Failed to copy app_config.toml to target\%BUILD_TYPE%\
    exit /b %ERRORLEVEL%
)

echo Build completed successfully.
echo Configuration file copied to target\%BUILD_TYPE%\app_config.toml

endlocal
