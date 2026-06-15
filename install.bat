@echo off
setlocal enabledelayedexpansion

REM Check for administrator privileges 
fltmc >nul 2>&1
if ERRORLEVEL 1 (
    echo The Packit install requires administrator privileges.
    choice /M "Do you wish to continue as administrator?"

    if ERRORLEVEL 2 (
        echo Packit installed cancelled
        exit /b 1
    )

    REM Rerun the script with elevated permissions (this will first prompt the user)
    powershell -Command "Start-Process cmd -Verb RunAs -ArgumentList '/k \"%~f0\"'"

    exit /b
)

set "VERSION=0.0.2"
set "REVISION=0"
if "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    set "CURRENT_OS=aarch64-pc-windows-msvc"
) else (
    set "CURRENT_OS=x86_64-pc-windows-msvc"
)

set "SOURCE_URL=https://github.com/pack-it/packit/releases/download/%VERSION%/packit@%VERSION%.tar.gz"
set "PREBUILD_URL=https://github.com/pack-it/packit/releases/download/%VERSION%/packit@%VERSION%-%REVISION%-%CURRENT_OS%.tar.gz"

REM Determine the prefix and config directory
set "PREFIX_DIR=C:\Program Files\packit"
set "CONFIG_DIR=C:\Program Files\packit"

REM Go into the prefix directory 
mkdir "%PREFIX_DIR%\packages\packit"
pushd "%PREFIX_DIR%\packages\packit"

REM Install Packit to the prefix directory 
echo Downloading Packit prebuild
curl --proto "=https" -sSfL %PREBUILD_URL% --output packit@%VERSION%-%REVISION%-%CURRENT_OS%.tar.gz
if not ERRORLEVEL 1 (
    tar -xf packit@%VERSION%-%REVISION%-%CURRENT_OS%.tar.gz
    del packit@%VERSION%-%REVISION%-%CURRENT_OS%.tar.gz
    ren packit@%VERSION%-%REVISION%-%CURRENT_OS% %VERSION%

    echo Downloaded prebuild
) else (
    set "answer="
    set /p "answer=Retrieving prebuilds failed. Do you wish to build Packit from source? (Y/n) "
    set "match="
    if "!answer!"=="n" set "match=1"
    if "!answer!"=="no" set "match=1"
    if "!match!"=="1" (
        echo Canceling installation of Packit
        popd
        exit /b 1
    )

    set RUSTUP_INSTALLED=0

    REM Make sure cargo exists before building Packit
    where cargo 2>nul >nul
    if ERRORLEVEL 1 (
        set "answer="
        set /p "answer=Cargo is not installed, do you wish to install it to build Packit? (y/N) "
        set "match="
        if "!answer!"=="n" set "match=1"
        if "!answer!"=="no" set "match=1"
        if "!answer!"=="" set "match=1"
        if "!match!"=="1" (
            echo Canceling installation of Packit
            popd
            exit /b 1
        )

        REM Install the correct rustup version for the current platform
        if "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
            curl --proto "=https" --tlsv1.2 -sSfL https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe --output rustup-init.exe
        ) else (
            if defined PROCESSOR_ARCHITEW6432 (
                curl --proto "=https" --tlsv1.2 -sSfL https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe --output rustup-init.exe
            ) else (
                curl --proto "=https" --tlsv1.2 -sSfL https://static.rust-lang.org/rustup/dist/i686-pc-windows-msvc/rustup-init.exe --output rustup-init.exe
            )
        )
        
        .\rustup-init.exe
        del .\rustup-init.exe

        REM Make sure that the rustup install was successful
        where cargo 2>nul >nul
        if ERRORLEVEL 1 (
            echo Installing rustup failed, canceling Packit installation
            popd
            exit /b 1
        )

        set RUSTUP_INSTALLED=1
    )

    curl --proto "=https" -sSfL %SOURCE_URL% --output packit@%VERSION%.tar.gz
    tar -xf packit@%VERSION%.tar.gz
    del packit@%VERSION%.tar.gz
    cd packit@%VERSION%
    cargo build-install --destination ..\$VERSION
    cd ..
    rmdir /s /q .\packit@%VERSION%

    if "!RUSTUP_INSTALLED!"==1 (
        set "answer="
        set /p "answer=You installed rustup to install Packit. This installation is not registered in Packit. Do you wish to uninstall it? (Y/n) "
        set "match="
        if "!answer!"=="y" set "match=1"
        if "!answer!"=="yes" set "match=1"
        if "!answer!"=="" set "match=1"
        if "!match!"=="1" (
            echo Uninstalling rustup
            rustup self uninstall
        )
    )
)

if not exist "%CONFIG_DIR%" (
    mkdir "%CONFIG_DIR%"
)

"%PREFIX_DIR%\packages\packit\%VERSION%\bin\packit.exe" init

REM Make sure that packit words
"%PREFIX_DIR%\bin\pit.exe" --version 2>nul >nul
if ERRORLEVEL 1 (
    echo Unsuccessfull install of Packit, the 'pit' command cannot be found
    popd
    exit /b 1
)

"%PREFIX_DIR%\bin\packit.exe" --version 2>nul >nul
if ERRORLEVEL 1 (
    echo Unsuccessfull install of Packit, the 'packit' command cannot be found
    popd
    exit /b 1
)

echo Successfully installed Packit

REM Exit early if Packit is already in the user PATH
echo ";%PATH%;" | find /I ";%PREFIX_DIR%\bin;" >nul
if %ERRORLEVEL%==0 (
    exit /b 0
)

REM Ask the user if they want to automatically add Packit to their PATH
set "answer="
set /p "answer=Do you wish to automatically add Packit to your user PATH? (Y/n) "
set "match="
if "!answer!"=="y" set "match=1"
if "!answer!"=="yes" set "match=1"
if "!answer!"=="" set "match=1"
if "!match!"=="1" (
    setx PATH "%PATH%;%PREFIX_DIR%\bin"
    popd
    exit /b 0
)

echo Add %PREFIX_DIR%\bin to your PATH by adding the command below to your shell:
echo setx PATH "%%PATH%%;%PREFIX_DIR%\bin"

popd
