#ifndef AppVersion
  #error AppVersion required
#endif
#ifndef BuildDirectory
  #error BuildDirectory required
#endif
[Setup]
AppId={{2EB43CF5-38FA-475E-97E8-6B32AB435AF0}
AppName=Aetheris
AppVersion={#AppVersion}
AppPublisher=Aetheris
DefaultDirName={autopf}\Aetheris
DisableDirPage=no
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..
OutputBaseFilename=aetheris-windows-x86_64-setup
SetupIconFile=..\resources\aetheris.ico
WizardStyle=modern
WizardImageBackColor=$211B16
DisableWelcomePage=no
CloseApplications=no
RestartApplications=no
AppMutex=Global\AetherisManagedActivity
SetupMutex=Global\AetherisInstaller
UninstallDisplayIcon={app}\aetheris-companion.exe
Compression=lzma2
SolidCompression=yes
LicenseFile=..\LICENSE
[Languages]
Name: french; MessagesFile: compiler:Languages\French.isl
Name: english; MessagesFile: compiler:Default.isl
[Tasks]
Name: startmenu; Description: "Ajouter au menu Démarrer"; Flags: checkedonce
Name: desktop; Description: "Créer un raccourci sur le Bureau"; Flags: unchecked
[Files]
Source: "{#BuildDirectory}\aetheris.exe"; DestDir: "{app}\service\installer-stage"; Flags: ignoreversion
Source: "{#BuildDirectory}\aetheris-companion.exe"; DestDir: "{app}\service\installer-stage"; Flags: ignoreversion
Source: "{#BuildDirectory}\aetheris-update-helper.exe"; DestDir: "{app}\service\installer-stage"; Flags: ignoreversion
Source: "{#BuildDirectory}\release.json"; DestDir: "{app}\service\installer-stage"; Flags: ignoreversion
Source: "{#BuildDirectory}\release.sig"; DestDir: "{app}\service\installer-stage"; Flags: ignoreversion
Source: "{#BuildDirectory}\aetheris-update-helper.exe"; DestDir: "{app}\service"; DestName: bootstrap.exe; Flags: ignoreversion
Source: "{#BuildDirectory}\aetheris-update-helper.exe"; Flags: dontcopy
[Dirs]
Name: "{app}\service"
[Registry]
Root: HKLM; Subkey: "SOFTWARE\Aetheris"; ValueType: string; ValueName: InstallDir; ValueData: "{app}"; Flags: uninsdeletekey
[Icons]
Name: "{commonprograms}\Aetheris"; Filename: "{app}\aetheris-companion.exe"; Tasks: startmenu
Name: "{commondesktop}\Aetheris"; Filename: "{app}\aetheris-companion.exe"; Tasks: desktop
[Run]
Filename: "{app}\aetheris-companion.exe"; Description: "Lancer Aetheris"; Flags: postinstall nowait skipifsilent runasoriginaluser
[UninstallDelete]
Type: files; Name: "{app}\aetheris.exe"
Type: files; Name: "{app}\aetheris-companion.exe"
Type: files; Name: "{app}\aetheris-update-helper.exe"
Type: files; Name: "{app}\release.json"
Type: files; Name: "{app}\release.sig"
Type: filesandordirs; Name: "{app}\service"
Type: filesandordirs; Name: "{app}\.aetheris-previous"
Type: filesandordirs; Name: "{app}\.aetheris-transaction"
[Code]
var
  Instances: Integer;
  Updates: Integer;
  CaptureStopped: Boolean;

function CreateFile(FileName: String; Access, Share: LongWord; Security: Integer; Creation, Attributes: LongWord; Template: Integer): Integer;
  external 'CreateFileW@kernel32.dll stdcall';
function CloseHandle(Handle: Integer): Boolean;
  external 'CloseHandle@kernel32.dll stdcall';

procedure CheckedExec(FileName, Parameters: String);
var Code: Integer;
begin
  if not Exec(FileName, Parameters, '', SW_HIDE, ewWaitUntilTerminated, Code) or (Code <> 0) then
    RaiseException('Opération impossible : ' + FileName + ' (code ' + IntToStr(Code) + ').');
end;

procedure ReleaseLocks;
begin
  if Instances >= 0 then begin CloseHandle(Instances); Instances := -1; end;
  if Updates >= 0 then begin CloseHandle(Updates); Updates := -1; end;
end;

function LockInstallation: Boolean;
begin
  Result := True;
  if FileExists(ExpandConstant('{app}\service\update.lock')) then begin
    Updates := CreateFile(ExpandConstant('{app}\service\update.lock'), $80000000, 4, 0, 3, 0, 0);
    if Updates < 0 then begin Result := False; exit; end;
    Instances := CreateFile(ExpandConstant('{app}\service\instances.lock'), $80000000, 4, 0, 3, 0, 0);
    if Instances < 0 then begin ReleaseLocks; Result := False; end;
  end;
end;

function InitializeSetup: Boolean;
begin
  Instances := -1; Updates := -1; CaptureStopped := False;
  Result := True;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var Existing: String;
begin
  Result := '';
  // A single registered installation; never silently move an existing service.
  if RegQueryStringValue(HKLM64, 'SOFTWARE\Aetheris', 'InstallDir', Existing) then
    if CompareText(Existing, ExpandConstant('{app}')) <> 0 then begin
      Result := 'Une installation existe dans ' + Existing + '. Désinstallez-la avant de changer de dossier.'; exit;
    end;
  ExtractTemporaryFile('aetheris-update-helper.exe');
  try
    CheckedExec(ExpandConstant('{tmp}\aetheris-update-helper.exe'), '--installer-check "' + ExpandConstant('{app}') + '"');
    CheckedExec(ExpandConstant('{tmp}\aetheris-update-helper.exe'), '--stop-service');
    ReleaseLocks;
    if not LockInstallation then begin
      Result := 'Fermez toutes les sessions Aetheris puis réessayez. Aucun processus ne sera arrêté de force.'; exit;
    end;
    CheckedExec(ExpandConstant('{tmp}\aetheris-update-helper.exe'), '--stop-capture-service');
    CaptureStopped := True;
    CheckedExec(ExpandConstant('{tmp}\aetheris-update-helper.exe'), '--installer-wait "' + ExpandConstant('{app}') + '"');
    CheckedExec(ExpandConstant('{tmp}\aetheris-update-helper.exe'), '--installer-recover "' + ExpandConstant('{app}') + '"');
    ForceDirectories(ExpandConstant('{app}'));
    CheckedExec(ExpandConstant('{sys}\icacls.exe'), '"' + ExpandConstant('{app}') + '" /setowner *S-1-5-32-544 /T /Q');
    CheckedExec(ExpandConstant('{sys}\icacls.exe'), '"' + ExpandConstant('{app}') + '" /inheritance:r /grant:r *S-1-5-18:(OI)(CI)F *S-1-5-32-544:(OI)(CI)F *S-1-5-32-545:(OI)(CI)RX /T /Q');
  except
    Result := GetExceptionMessage;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then begin
    // Preserve state when repairing. Programs and service state inherit a protected ACL.
    if not FileExists(ExpandConstant('{app}\service\instances.lock')) then
      SaveStringToFile(ExpandConstant('{app}\service\instances.lock'), '', False);
    if not FileExists(ExpandConstant('{app}\service\update.lock')) then
      SaveStringToFile(ExpandConstant('{app}\service\update.lock'), '', False);
    CheckedExec(ExpandConstant('{app}\service\installer-stage\aetheris-update-helper.exe'), '--installer-apply');
    CheckedExec(ExpandConstant('{sys}\icacls.exe'), '"' + ExpandConstant('{app}') + '" /setowner *S-1-5-32-544 /T /Q');
    CheckedExec(ExpandConstant('{app}\aetheris-update-helper.exe'), '--configure-service');
    ReleaseLocks;
    CheckedExec(ExpandConstant('{sys}\sc.exe'), 'start AetherisUpdate');
    CheckedExec(ExpandConstant('{sys}\sc.exe'), 'start AetherisCapture');
    CaptureStopped := False;
  end;
end;

procedure DeinitializeSetup;
var Code: Integer;
begin
  ReleaseLocks;
  // An aborted repair must not leave the existing capture service stopped.
  if CaptureStopped then
    Exec(ExpandConstant('{sys}\sc.exe'), 'start AetherisCapture', '', SW_HIDE, ewWaitUntilTerminated, Code);
end;

function InitializeUninstall: Boolean;
begin
  Instances := -1; Updates := -1;
  Result := False;
  try
    CheckedExec(ExpandConstant('{app}\aetheris-update-helper.exe'), '--stop-service');
    if not LockInstallation then begin
      MsgBox('Fermez toutes les sessions Aetheris puis réessayez.', mbError, MB_OK); exit;
    end;
    CheckedExec(ExpandConstant('{sys}\sc.exe'), 'delete AetherisUpdate');
    CheckedExec(ExpandConstant('{app}\aetheris-update-helper.exe'), '--remove-capture-service');
    Result := True;
  except
    MsgBox(GetExceptionMessage, mbError, MB_OK);
  end;
end;

procedure DeinitializeUninstall;
begin
  ReleaseLocks;
end;
