@ECHO OFF
cls
setlocal enabledelayedexpansion

for /f "skip=1 delims=" %%i in ('wmic computersystem get model') do (
if %%i GTR 0 set model=%%i
)

for /f "skip=1 delims=" %%i in ('wmic computersystem get manufacturer') do (
if %%i GTR 0 set manufacturer=%%i
)
if "%manufacturer: =%" == "LENOVO" (
for /f "skip=1 delims=" %%i in ('wmic computersystem get systemfamily') do (
if %%i GTR 0 set model=%%i
)
)

for /f "skip=1 delims=" %%i in ('wmic bios get serialnumber') do (
if %%i GTR 0 set serial=%%i
)

title %model% %serial%

:: if exist "C:\Windows\System32\igfxtray.exe" (
:: REG ADD "HKLM\Software\INTEL\DISPLAY\IGFXCUI\HotKeys" /v "Enable" /t REG_DWORD /d 00000000 /f
:: )

net stop WMPNetworkSvc

sc config wmpnetworksvc start= disabled

for /f "skip=1 delims=" %%i in ('wmic bios get smbiosbiosversion') do (
if %%i GTR 0 (
set bios=%%i
)
)


ECHO ===Sync Time=====================================================
ECHO.
sc config w32time start= auto
net start w32time
w32tm /config /update /manualpeerlist:time.google.com
w32tm /resync /force
w32tm /resync /force
w32tm /resync /force

ECHO ===Hardware Info=================================================
if "%model:~0,15%" == "HPEliteBook8530" (
ECHO.
ECHO BIOS Version: !bios!
ECHO.
)
if "%model:~0,15%" == "HPEliteBook8540" (
ECHO.
ECHO BIOS Version: !bios!
ECHO.
)

ECHO Memory Information:
for /f "skip=1" %%i in ('wmic memorychip get capacity') do (
if %%i GTR 0 (
set tempram=%%i
set tempram=!tempram:~0,-6!
if !tempram! GTR 1000 (echo !tempram:~0,-3! GB) else if !tempram! GTR 500 (echo 512 MB) else (echo. > nul)
)
)

ECHO.

for /f "skip=1 delims=" %%i in ('wmic diskdrive get size') do (
if %%i GTR 0 (
set disksize=%%i
set gigs=Hard Drive Size: !disksize:~0,-12! GB
echo !gigs!
)
)

ECHO.

for /f "skip=1 delims=" %%i in ('wmic cpu get name') do (
if %%i GTR 0 (
set cpuname=%%i
for /f "delims=@" %%a in ("!cpuname!") do set first=%%a
)
)

for /f "skip=1 delims=" %%i in ('wmic cpu get numberofcores') do (
if %%i GTR 0 (
set numcores=%%i
set numcores=!numcores: =!
)
)

for /f "skip=1 delims=" %%i in ('wmic cpu get maxclockspeed') do (
if %%i GTR 0 (
set /a clockspeed=%%i
set clockspeed=!clockspeed:~0,-3!.!clockspeed:~-3!
set clockspeed=!clockspeed:~0,-1! GHz
)
)

echo %first%with %numcores% cores at %clockspeed%

ECHO.
:: replace
start chrome.exe -incognito --new-window "http://192.168.1.117:8000/qc_form.html?download_id_on_save&makemodel=%model%&oemserial=%serial%&operatingsystem=Windows10"
pause

exit

