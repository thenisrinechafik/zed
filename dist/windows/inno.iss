; Zed Windows Installer Script (Inno Setup)
; Generated for Public Beta RC workflows. This script is channel-aware and
; consumes environment variables provided by scripts/win/package.ps1.

#define AppChannel GetEnv("ZED_CHANNEL")
#define AppVersion GetEnv("ZED_VERSION")
#define InstallDirName \
    Str(CompareText(AppChannel, "stable") == 0 ? "Zed" : \
        (CompareText(AppChannel, "preview") == 0 ? "Zed Preview" : "Zed Nightly"))

[Setup]
AppName=Zed
AppVersion={#AppVersion}
AppId={{A92C29A0-4E83-4E75-9A47-9E57E4C2{#InstallDirName}}}
DefaultDirName={pf64}\{#InstallDirName}
OutputBaseFilename=Zed-{#AppChannel}-{#AppVersion}
SetupLogging=yes
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=lowest
Compression=lzma
SolidCompression=yes
WizardStyle=modern
DisableProgramGroupPage=yes
ChangesAssociations=yes
ChangesEnvironment=yes
UninstallDisplayName={#InstallDirName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "..\..\target\release\zed.exe"; DestDir: "{app}"; Flags: ignoreversion signonce
Source: "..\..\dist\windows\redistributables\*"; DestDir: "{app}\redist"; Flags: recursesubdirs createallsubdirs
Source: "..\..\LICENSE*"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autodesktop}\{#InstallDirName}"; Filename: "{app}\zed.exe"; Tasks: desktopicon
Name: "{group}\{#InstallDirName}"; Filename: "{app}\zed.exe"

[Registry]
Root: HKCU; Subkey: "Software\Classes\zed"; ValueType: string; ValueData: "URL:Zed Protocol"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\zed"; ValueName: "URL Protocol"; ValueType: string; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\zed\shell\open\command"; ValueType: string; ValueData: '"{app}\zed.exe" "%1"'

[Run]
Filename: "{app}\zed.exe"; Description: "Launch {#InstallDirName}"; Flags: nowait postinstall skipifsilent

[Code]
procedure InitializeWizard;
begin
  Log(Format('Launching installer for channel %s (version %s)', [GetEnv('ZED_CHANNEL'), GetEnv('ZED_VERSION')]));
end;
