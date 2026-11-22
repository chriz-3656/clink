!define PRODUCT_NAME "Clink"
!ifndef PRODUCT_VERSION
  !define PRODUCT_VERSION "1.0.7"
!endif
!define PRODUCT_PUBLISHER "chriz-3656"
!define PRODUCT_WEB_SITE "https://github.com/chriz-3656/clink"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\clink.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "clink-setup-v${PRODUCT_VERSION}-windows-x86_64.exe"
InstallDir "$PROGRAMFILES64\Clink"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show
RequestExecutionLevel admin

Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  SetOverwrite ifnewer
  File "/oname=clink.exe" "..\..\dist\clink-windows-x86_64.exe"
  CreateDirectory "$SMPROGRAMS\Clink"
  CreateShortCut "$SMPROGRAMS\Clink\Clink.lnk" "$INSTDIR\clink.exe"
  CreateShortCut "$DESKTOP\Clink.lnk" "$INSTDIR\clink.exe"
SectionEnd

Section -Post
  WriteUninstaller "$INSTDIR\uninst.exe"
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\clink.exe"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninst.exe"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
SectionEnd

Section Uninstall
  Delete "$DESKTOP\Clink.lnk"
  Delete "$SMPROGRAMS\Clink\Clink.lnk"
  RMDir "$SMPROGRAMS\Clink"
  Delete "$INSTDIR\clink.exe"
  Delete "$INSTDIR\uninst.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "${PRODUCT_UNINST_KEY}"
  DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
SectionEnd
