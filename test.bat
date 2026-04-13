@echo off

for /L %%X in (0,1,100) do (
    cargo run uninstall make
    cargo run install make
    if ERRORLEVEL 1 (
        echo Failed
        exit /b 1
    )
)