@echo off
setlocal enabledelayedexpansion

REM A function to handle (yes/no) questions.
REM Returns 1 if the default option was chosen, 0 otherwise.
goto ask_end
:ask
REM The preferred answer to the question (the default), needs to be a string of value 'Y' or 'N'
set "PREFERRED=%~1"

REM The question to ask
set "QUESTION=%~2"

if "!PREFERRED!"=="Y" (
    set /p "answer=!QUESTION! (Y/n) "
    if /I "!answer!"=="N" exit /b 0
    if /I "!answer!"=="NO" exit /b 0
    exit /b 1
)

set /p "answer=!QUESTION! (y/N) "
if /I "!answer!"=="Y" exit /b 0
if /I "!answer!"=="YES" exit /b 0
exit /b 1

:ask_end

REM Removes all created files in case of an error and exits with the appropriate status code.
goto cleanup_end
:cleanup
REM Set the status code to the errorlevel, if the errorlevel is zero set it to 1 (cleanup only happens in case of error)
set STATUS_CODE=!ERRORLEVEL!
if !STATUS_CODE!==0 set STATUS_CODE=1

popd
echo Removing installed Packit files

REM Remove the prefix directory if `PREFIX_DIR` exists
if exist "%PREFIX_DIR%" (
    rmdir /s /q "%PREFIX_DIR%"
)

REM Remove the config directory if `PREFIX_DIR` exists
if exist "%CONFIG_DIR%" (
    rmdir /s /q "%CONFIG_DIR%"
)

exit /b %STATUS_CODE%
:cleanup_end

set "VERSION=0.0.4"
set "REVISION=0"

echo Installing Packit %VERSION% (%REVISION%)
echo Current OS: Windows

if "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    set "TARGET=aarch64-pc-windows-msvc"
) else (
    set "TARGET=x86_64-pc-windows-msvc"
)

echo Current target: %TARGET%

set "SOURCE_URL=https://github.com/pack-it/packit/releases/download/%VERSION%/packit@%VERSION%.tar.gz"
set "PREBUILD_URL=https://github.com/pack-it/packit/releases/download/%VERSION%/packit@%VERSION%-%REVISION%-%TARGET%.tar.gz"

REM Determine the prefix and config directory
set "PREFIX_DIR=C:\Program Files\packit"
set "CONFIG_DIR=C:\Program Files\packit"

echo Prefix directory: %PREFIX_DIR%
echo Config directory: %CONFIG_DIR%

REM Ask the user for administrator rights
call :ask "Y" "The Packit install script requires administrator privileges to modify '%PREFIX_DIR%' and '%CONFIG_DIR%', do you wish to continue"
if not ERRORLEVEL 1 (
    echo Canceling installation of Packit
    exit /b 1
)

REM Check for administrator privileges 
fltmc >nul 2>&1
if ERRORLEVEL 1 (
    REM Rerun the script with elevated permissions (this will first prompt the user)
    REM Change directory to the current directory, so the working directory will not be 'windows\system32'
    powershell -Command "Start-Process cmd -Verb RunAs -ArgumentList '/k cd /d %~dp0 && \"%~f0\"'"

    exit /b
)

REM Exit early with code 0 if there already is a version of Packit installed
REM Note that we can't rely on the `packit init` command, because we don't know if it fails because of an already existing config file
if exist "%CONFIG_DIR%\Config.toml" (
    echo Packit already seems to be installed, config file found in '%CONFIG_DIR%'
    exit /b 0
)
if exist "%PREFIX_DIR%\Register.toml" (
    echo Packit already seems to be installed, register file found in '%PREFIX_DIR%'
    exit /b 0
)

REM Go into the prefix directory 
mkdir "%PREFIX_DIR%\packages\packit"
if ERRORLEVEL 1 goto cleanup
pushd "%PREFIX_DIR%\packages\packit"
if ERRORLEVEL 1 goto cleanup

REM Install Packit to the prefix directory 
echo Downloading Packit prebuild from `%PREBUILD_URL%`
curl --proto "=https" -sSfL "%PREBUILD_URL%" --output packit@%VERSION%-%REVISION%-%TARGET%.tar.gz
if not ERRORLEVEL 1 (
    tar -xf packit@%VERSION%-%REVISION%-%TARGET%.tar.gz
    if ERRORLEVEL 1 goto cleanup
    del packit@%VERSION%-%REVISION%-%TARGET%.tar.gz
    if ERRORLEVEL 1 goto cleanup
    ren packit@%VERSION%-%REVISION%-%TARGET% %VERSION%
    if ERRORLEVEL 1 goto cleanup

    echo Downloading Packit prebuild successful
) else (
    REM Check internet connection with reliable site
    curl -sSf http://www.google.com >NUL 2>&1
    if ERRORLEVEL 1 (
        echo Retrieving prebuilds failed, because there is no working internet connection
        echo Canceling installation of Packit
        goto cleanup
    )

    call :ask "Y" "Retrieving prebuilds failed. Do you wish to build Packit from source"
    if not ERRORLEVEL 1 (
        echo Canceling installation of Packit
        goto cleanup
    )

    set RUSTUP_INSTALLED=0

    REM Make sure cargo exists before building Packit
    where cargo 2>nul >nul
    if ERRORLEVEL 1 (
        call :ask "N" "Cargo is not installed, do you wish to install it to build Packit"
        if ERRORLEVEL 1 (
            echo Canceling installation of Packit
            goto cleanup
        )

        REM Check if vswhere exists
        if not exist "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe" (
            echo Cannot find vswhere to check for 'link.exe' existence
            goto cleanup
        )

        if !TARGET!=="aarch64-pc-windows-msvc" (
            set "MSVC_TARGET=ARM64"
        ) else (
            set "MSVC_TARGET=x86.x64"
        )

        REM Search MSVC build tools using vswhere
        for /f "delims=" %%A in ('"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath -products * -requires Microsoft.VisualStudio.Component.VC.Tools.!MSVC_TARGET!') do set "VS_PATH=%%A"

        REM Check if MSVC exists (and thus link.exe)
        if not exist !VS_PATH! (
            echo MSVC cannot be found, 'link.exe' is needed to use cargo for the Packit build
            goto cleanup
        )

        REM Choose the correct rustup version for the current platform
        if "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
            set "RUSTUP_URL=https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe"
        ) else if "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
            set "RUSTUP_URL=https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
        ) else (
            echo Current target not supported
            goto cleanup
        )

        REM Check for empty USERPROFILE
        if "%USERPROFILE%"=="" (
            echo Cannot do rustup install, because USERPROFILE variable is empty
            goto cleanup
        )

        REM Install cargo
        echo Installing cargo from '!RUSTUP_URL!'
        set "RUSTUP_HOME=%USERPROFILE%\.rustup"
        set "CARGO_HOME=%USERPROFILE%\.cargo"
        curl --proto "=https" --tlsv1.2 -sSfL "!RUSTUP_URL!" --output rustup-init.exe
        if ERRORLEVEL 1 goto cleanup
        
        .\rustup-init.exe -y
        if ERRORLEVEL 1 goto cleanup
        del .\rustup-init.exe
        if ERRORLEVEL 1 goto cleanup

        REM Make sure that the rustup install was successful
        !CARGO_HOME!\bin\cargo.exe --version 2>nul >nul 
        if ERRORLEVEL 1 (
            echo Installing rustup failed, canceling Packit installation
            goto cleanup
        )

        echo Installing cargo successful
        set RUSTUP_INSTALLED=1
    )

    echo Downloading Packit source files from '%SOURCE_URL%'
    curl --proto "=https" -sSfL %SOURCE_URL% --output packit@%VERSION%.tar.gz
    if ERRORLEVEL 1 goto cleanup
    echo Downloading Packit source files successful

    echo Unpacking Packit source files
    tar -xf packit@%VERSION%.tar.gz
    if ERRORLEVEL 1 goto cleanup
    echo Unpacking Packit source files successful

    del packit@%VERSION%.tar.gz
    if ERRORLEVEL 1 goto cleanup
    cd packit@%VERSION%
    if ERRORLEVEL 1 goto cleanup

    echo Building Packit from source
    "!CARGO_HOME!\bin\cargo.exe" build-install --destination ..\%VERSION%
    if ERRORLEVEL 1 goto cleanup
    cd ..
    if ERRORLEVEL 1 goto cleanup
    rmdir /s /q .\packit@%VERSION%
    if ERRORLEVEL 1 goto cleanup

    REM Remove rustup after building Packit
    if "!RUSTUP_INSTALLED!"=="1" (
        echo Uninstalling rustup
        "!CARGO_HOME!\bin\rustup.exe" self uninstall -y
        if ERRORLEVEL 1 goto cleanup
        echo Uninstalling rustup successful
    )

    echo Building Packit from source successful
)

if not exist "%CONFIG_DIR%" (
    mkdir "%CONFIG_DIR%"
    if ERRORLEVEL 1 goto cleanup
)

echo Initializing Packit
"%PREFIX_DIR%\packages\packit\%VERSION%\bin\packit.exe" init
if ERRORLEVEL 1 goto cleanup
echo Initializing Packit successful

REM Remove readonly attributes on Packit directories
attrib -R /S /D "*"

REM Make sure that packit words
echo Testing Packit install
"%PREFIX_DIR%\bin\pit.exe" --version 2>nul >nul
if ERRORLEVEL 1 (
    echo Unsuccessfull install of Packit, the 'pit' command cannot be found
    goto cleanup
)

"%PREFIX_DIR%\bin\packit.exe" --version 2>nul >nul
if ERRORLEVEL 1 (
    echo Unsuccessfull install of Packit, the 'packit' command cannot be found
    goto cleanup
)

echo Successfully installed Packit!

REM Get the PATH
for /f "delims=" %%A in ('
    powershell -NoProfile -Command "[Environment]::GetEnvironmentVariable('Path', 'User')"
') do set "ORIGINAL_PATH=%%A"

REM Exit early if Packit is already in the user PATH
echo ";%ORIGINAL_PATH%;" | find /I ";%PREFIX_DIR%\bin;" >nul
if %ERRORLEVEL%==0 (
    echo Packit already found in PATH, no further actions should be required
    popd
    exit /b 0
)

REM Ask the user if they want to automatically add Packit to their PATH
call :ask "Y" "Do you wish to automatically add Packit to your user PATH"
if ERRORLEVEL 1 (
    REM Determine if there should be a semicolon prefix
    set "SEMICOLON_PREFIX =;"
    if "!ORIGINAL_PATH:~-1!"==";" (
        set "SEMICOLON_PREFIX ="
    )

    powershell -NoProfile -Command "[Environment]::SetEnvironmentVariable('Path', '!ORIGINAL_PATH!!SEMICOLON_PREFIX!%PREFIX_DIR%\bin;', 'User')"
    if ERRORLEVEL 1 goto cleanup
    echo Restart your shell to refresh your path and use Packit
    popd
    exit /b 0
)

echo Add '%PREFIX_DIR%\bin' to your PATH to start working with Packit

popd
