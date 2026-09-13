@echo off
setlocal

set "ROOT=%~dp0"
pushd "%ROOT%" >nul

set "SOURCE=%~1"
if "%SOURCE%"=="" set "SOURCE=examples\compiled_runtime.apl"

set "OUT_DIR=%~2"
if "%OUT_DIR%"=="" (
  for %%F in ("%SOURCE%") do set "OUT_DIR=build\%%~nF_bat"
)
for %%F in ("%SOURCE%") do set "EXE_NAME=%%~nF_compiled.exe"

echo [APL] compile "%SOURCE%" -^> "%OUT_DIR%"
cargo run -p apl -- compile "%SOURCE%" "%OUT_DIR%"
if errorlevel 1 (
  popd >nul
  exit /b 1
)

echo.
echo [APL] done
echo [APL] exe: "%OUT_DIR%\target\debug\%EXE_NAME%"

popd >nul
endlocal
