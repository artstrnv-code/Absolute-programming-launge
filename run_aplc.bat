@echo off
setlocal

set "ROOT=%~dp0"
pushd "%ROOT%" >nul

set "IR_FILE=%~1"
if "%IR_FILE%"=="" set "IR_FILE=build\compiled_runtime.aplc"

echo [APL] run-ir "%IR_FILE%"
cargo run -p apl -- run-ir "%IR_FILE%"

set "STATUS=%ERRORLEVEL%"
popd >nul
endlocal & exit /b %STATUS%
