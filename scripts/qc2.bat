@ECHO OFF

set qc_sheet_path=%homedrive%%homepath%\Downloads\qc_form.txt

echo Loading ID from: %qc_sheet_path%

if exist %qc_sheet_path% (
echo Found Form ID: %qc_sheet_id%
set /p qc_sheet_id=<"%qc_sheet_path%"
start chrome.exe -incognito --new-window "http://192.168.1.117:8000/qc_form.html?id=%qc_sheet_id%"
del "%qc_sheet_path%"
) else (
echo CANNOT FILE FILE
)

echo Press any key after pdf is saved
pause