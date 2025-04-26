@echo off
echo Cleaning Rust build artifacts...

echo Running cargo clean...
cargo clean

if %ERRORLEVEL% neq 0 (
    echo Failed to run 'cargo clean'
    exit /b %ERRORLEVEL%
)

echo Removing target directory...
if exist target (
    rd /s /q target
    if %ERRORLEVEL% neq 0 (
        echo Failed to remove target directory
        exit /b %ERRORLEVEL%
    )
    echo Target directory removed successfully.
) else (
    echo Target directory not found. Nothing to remove.
)

echo Cleanup completed successfully.
