@echo off
setlocal

if defined VCToolsInstallDir if exist "%VCToolsInstallDir%bin\Hostx64\x64\link.exe" (
  "%VCToolsInstallDir%bin\Hostx64\x64\link.exe" %*
  exit /b %errorlevel%
)

set "MSVC_TOOLS="
for %%Y in (2022 2019) do (
  for %%E in (Community Professional Enterprise BuildTools) do (
    if exist "%ProgramFiles%\Microsoft Visual Studio\%%Y\%%E\VC\Tools\MSVC" (
      for /f "delims=" %%V in ('dir /b /ad-h /o-n "%ProgramFiles%\Microsoft Visual Studio\%%Y\%%E\VC\Tools\MSVC"') do (
        set "MSVC_TOOLS=%ProgramFiles%\Microsoft Visual Studio\%%Y\%%E\VC\Tools\MSVC\%%V"
        goto have_msvc_tools
      )
    )
  )
)

echo Could not locate the MSVC tools directory. 1>&2
exit /b 1

:have_msvc_tools
set "WINDOWS_SDK_LIB="
for /f "delims=" %%V in ('dir /b /ad-h /o-n "%ProgramFiles(x86)%\Windows Kits\10\Lib"') do (
  set "WINDOWS_SDK_LIB=%ProgramFiles(x86)%\Windows Kits\10\Lib\%%V"
  goto have_windows_sdk
)

echo Could not locate the Windows 10 SDK libraries. 1>&2
exit /b 1

:have_windows_sdk
if not exist "%MSVC_TOOLS%\bin\Hostx64\x64\link.exe" (
  echo Could not locate MSVC link.exe under "%MSVC_TOOLS%". 1>&2
  exit /b 1
)

set "LIB=%MSVC_TOOLS%\lib\onecore\x64;%WINDOWS_SDK_LIB%\ucrt\x64;%WINDOWS_SDK_LIB%\um\x64;%LIB%"
"%MSVC_TOOLS%\bin\Hostx64\x64\link.exe" %*
exit /b %errorlevel%
