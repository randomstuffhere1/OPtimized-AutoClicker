@echo off
echo Building debug...
cargo build
set debug_success=%errorlevel%

echo Building release...
cargo build --release
set release_success=%errorlevel%

if %debug_success% equ 0 if %release_success% equ 0 (
    echo All builds passed
) else (
    echo Some builds failed
)