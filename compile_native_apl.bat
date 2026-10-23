@echo off
setlocal

set "ROOT=%~dp0"
pushd "%ROOT%" >nul

set "SOURCE=%~1"
if "%SOURCE%"=="" set "SOURCE=examples\linked_hello.apl"

set "OUT_DIR=%~2"
if "%OUT_DIR%"=="" (
  for %%F in ("%SOURCE%") do set "OUT_DIR=build\%%~nF_native"
)
for %%F in ("%SOURCE%") do set "EXE_NAME=%%~nF_compiled_native.exe"

echo [APL] compile-native "%SOURCE%" -^> "%OUT_DIR%"
cargo run -p apl -- compile-native "%SOURCE%" "%OUT_DIR%"
if errorlevel 1 (
  popd >nul
  exit /b 1
)

echo.
echo [APL] done
echo [APL] assembly: "%OUT_DIR%\program.s"
echo [APL] exe: "%OUT_DIR%\target\debug\%EXE_NAME%"

popd >nul
endlocal
