@echo off
echo Checking debug...
cargo check
set debug_success=%errorlevel%

echo Checking release...
cargo check --release
set release_success=%errorlevel%

if %debug_success% equ 0 if %release_success% equ 0 (
    echo All checks passed
) else (
    echo Some checks failed
)