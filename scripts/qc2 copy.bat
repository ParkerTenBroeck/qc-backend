@ECHO OFF

set qc_sheet_path=%homedrive%%homepath%\Downloads\qc_form.txt
echo Loading ID from: %qc_sheet_path%

if exist %qc_sheet_path% (
    GoTo QC_FORM_FOUND
) else (
    GoTo QC_FORM_NEW
)

:QC_FORM_FOUND
echo Found Form ID: %qc_sheet_id%
set /p qc_sheet_id=<"%qc_sheet_path%"
start chrome.exe -incognito --new-window "http://192.168.1.117:8000/qc_form.html/%qc_sheet_id%"
GoTo END_QC_FORM

:QC_FORM_NEW
echo No Existing Form, Starting new
start chrome.exe -incognito --new-window "http://192.168.1.117:8000/qc_form.html?download_id_on_save&make_model=%model%&oem_serial=%serial%&operating_system=win10"
GoTo END_QC_FORM

:END_QC_FORM

echo Press any key after pdf is saved
pause