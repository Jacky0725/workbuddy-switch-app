; Remove only this product's startup approval entry on a full uninstall.
; The default Tauri template removes the matching HKCU Run value.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "WorkBuddy Manager"
  ${EndIf}
!macroend
