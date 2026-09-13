@echo off
setlocal

set "ROOT=%~dp0"
pushd "%ROOT%" >nul

set "SOURCE=%~1"
if "%SOURCE%"=="" set "SOURCE=examples\compiled_runtime.apl"

set "OUT_FILE=%~2"
if "%OUT_FILE%"=="" (
  for %%F in ("%SOURCE%") do set "OUT_FILE=build\%%~nF.aplc"
)

echo [APL] emit "%SOURCE%" -^> "%OUT_FILE%"
cargo run -p apl -- emit "%SOURCE%" "%OUT_FILE%"
if errorlevel 1 (
  popd >nul
  exit /b 1
)

echo.
echo [APL] done
echo [APL] ir: "%OUT_FILE%"

popd >nul
endlocal
