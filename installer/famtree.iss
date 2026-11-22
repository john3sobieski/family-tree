; Inno Setup script for FamTree. Build: ISCC /DAppVersion=0.1.0 installer\famtree.iss
; Per-user install (no admin rights). One Setup for Windows 10 and 11.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#define AppName "FamTree"
#define AppExe "famtree.exe"

[Setup]
AppId={{D115EF89-C89A-4FF8-B9E2-C9E8511619F6}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=john3sobieski
AppPublisherURL=https://github.com/john3sobieski/family-tree
AppSupportURL=https://github.com/john3sobieski/family-tree/issues
AppUpdatesURL=https://github.com/john3sobieski/family-tree/releases
VersionInfoVersion={#AppVersion}
MinVersion=10.0
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
SetupIconFile=..\assets\icon.ico
OutputDir=..\dist
OutputBaseFilename=FamTree-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ChangesAssociations=yes
CloseApplications=yes
CloseApplicationsFilter={#AppExe}
ShowLanguageDialog=auto
UsePreviousLanguage=no

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "pl"; MessagesFile: "compiler:Languages\Polish.isl"

[CustomMessages]
en.AssocTask=Open .famtree files with FamTree
ru.AssocTask=Открывать файлы .famtree в FamTree
es.AssocTask=Abrir archivos .famtree con FamTree
fr.AssocTask=Ouvrir les fichiers .famtree avec FamTree
de.AssocTask=.famtree-Dateien mit FamTree öffnen
pl.AssocTask=Otwieraj pliki .famtree w FamTree
en.FileDesc=FamTree family tree
ru.FileDesc=Семейное дерево FamTree
es.FileDesc=Árbol genealógico de FamTree
fr.FileDesc=Arbre généalogique FamTree
de.FileDesc=FamTree-Stammbaum
pl.FileDesc=Drzewo genealogiczne FamTree

[Tasks]
Name: "assoc"; Description: "{cm:AssocTask}"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked

[Files]
Source: "..\target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

; Same keys as src/assoc.rs, so the in-app button and the installer agree.
[Registry]
Root: HKCU; Subkey: "Software\Classes\.famtree"; ValueType: string; ValueName: ""; ValueData: "FamTree.File"; Flags: uninsdeletekey; Tasks: assoc
Root: HKCU; Subkey: "Software\Classes\.famtree\OpenWithProgids"; ValueType: none; ValueName: "FamTree.File"; Flags: uninsdeletevalue; Tasks: assoc
Root: HKCU; Subkey: "Software\Classes\FamTree.File"; ValueType: string; ValueName: ""; ValueData: "{cm:FileDesc}"; Flags: uninsdeletekey; Tasks: assoc
Root: HKCU; Subkey: "Software\Classes\FamTree.File\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"",0"; Tasks: assoc
Root: HKCU; Subkey: "Software\Classes\FamTree.File\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: assoc

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Language setting saved by the app; trees (.famtree files) are never touched.
Type: filesandordirs; Name: "{userappdata}\FamTree"
