# tools/test-reaper-send.ps1
# Prove a local ReaScript bridge in an isolated REAPER instance.
# No user projects, source sounds or normal REAPER preferences are used.
#Requires -Version 7.3
[CmdletBinding()]
param(
    [string]$ReaperPath = 'C:\Program Files\REAPER (x64)\reaper.exe',
    [switch]$Run,
    [switch]$Interactive,
    [switch]$TestFocus,
    [switch]$PrepareProfile,
    [switch]$SkipFocus,
    [switch]$KeepOpen,
    [ValidatePattern('^[0-9a-f]{32}$')][string]$ResumeRun,
    [ValidatePattern('^[0-9a-f]{32}$')][string]$SeedRun,
    [ValidateRange(10, 600)][int]$StartupTimeoutSeconds = 120
)
$ErrorActionPreference = 'Stop'
# Hidden GUI processes can still display dialogs, acquire devices or steal input.
# Fail before filesystem preparation or process launch on the shared desktop.
if (($Run -or $ResumeRun) -and !$Interactive) {
    throw 'REAPER test deferred: -Interactive requires an agreed idle desktop or an isolated session. No REAPER process was launched.'
}
if ($TestFocus -and (!$Interactive -or (!$Run -and !$ResumeRun))) {
    throw 'Auto-Focus testing requires -Run/-ResumeRun, -Interactive and an agreed interactive test window.'
}
if ($TestFocus -and $SkipFocus) { throw 'TestFocus and SkipFocus are mutually exclusive.' }
if ($PrepareProfile -and ($Run -or $ResumeRun -or $TestFocus)) { throw 'PrepareProfile only prepares files; use it separately from native testing.' }
if (!(Test-Path -LiteralPath $ReaperPath -PathType Leaf)) { throw "REAPER not found: $ReaperPath" }
$taskRoot = Split-Path -Parent $PSScriptRoot
$profileRoot = Join-Path $taskRoot '.rust-validation/reaper-profile'
$ini = Join-Path $profileRoot 'reaper.ini'
if (!$Run -and !$ResumeRun -and !$PrepareProfile) {
    Write-Output "REAPER $((Get-Item -LiteralPath $ReaperPath).VersionInfo.FileVersion): $ReaperPath"
    Write-Output "Persistent test profile: $profileRoot"
    Write-Output 'Read-only preflight. -PrepareProfile prepares files without opening REAPER.'
    Write-Output 'Native tests require -Run -Interactive; Auto-Focus also requires -TestFocus.'
    return
}
New-Item -ItemType Directory -Path $profileRoot -Force | Out-Null
$profileLease = [IO.File]::Open((Join-Path $profileRoot 'profile.lock'), [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
try {
$activeInstancePath = Join-Path $profileRoot 'active-instance.json'
if (Test-Path -LiteralPath $activeInstancePath) {
    $activeInstance = Get-Content -LiteralPath $activeInstancePath -Raw | ConvertFrom-Json
    $activeProcess = Get-Process -Id $activeInstance.pid -ErrorAction SilentlyContinue
    if ($activeProcess -and $activeProcess.StartTime.ToUniversalTime().Ticks -eq $activeInstance.start_ticks -and
        (!$ResumeRun -or $activeInstance.run_id -cne $ResumeRun)) {
        throw 'The dedicated REAPER test profile is already in use. Resume its run instead of launching another instance.'
    }
}
# Reuse the same settings/cache between tests. Per-run WAVs, projects and queues
# remain separate. Never copy registration files or reset an existing profile.
if (!(Test-Path -LiteralPath $ini)) {
    if ($SeedRun) {
        $seedRoot = Join-Path $taskRoot ('.rust-validation/reaper-send/' + $SeedRun)
        $seedInstance = Get-Content -LiteralPath (Join-Path $seedRoot 'instance.json') -Raw | ConvertFrom-Json
        $seedResult = Get-Content -LiteralPath (Join-Path $seedRoot 'result.json') -Raw | ConvertFrom-Json
        if ($seedResult.insertion -cne 'passed' -or $seedResult.delivered_bytes -cne 'identical') {
            throw 'Only a verified owned test run can seed the persistent profile.'
        }
        $seedProcess = Get-Process -Id $seedInstance.pid -ErrorAction SilentlyContinue
        if ($seedProcess -and $seedProcess.StartTime.ToUniversalTime().Ticks -eq $seedInstance.start_ticks) {
            throw 'Close the owned seed instance normally before reusing its profile.'
        }
        foreach ($name in @('reaper.ini','reaper-install-rev.txt','reaper-vstplugins64.ini','reaper-vstshells64.ini','reaper-fxtags.ini')) {
            $sourceCache = Join-Path $seedRoot $name
            if (Test-Path -LiteralPath $sourceCache -PathType Leaf) { Copy-Item -LiteralPath $sourceCache -Destination (Join-Path $profileRoot $name) }
        }
        $settings = [IO.File]::ReadAllText($ini)
        $settings = [regex]::Replace($settings, '(?m)^loadlastproj=.*$', 'loadlastproj=0')
        [IO.File]::WriteAllText($ini, $settings, [Text.UTF8Encoding]::new($false))
    } else {
        [IO.File]::WriteAllText($ini, "[REAPER]`r`nshowlastproj=0`r`nloadlastproj=0`r`n", [Text.UTF8Encoding]::new($false))
    }
} elseif ($SeedRun) {
    throw 'The persistent profile already exists; omit SeedRun. Existing settings are preserved.'
}
if ($PrepareProfile) {
    Write-Output "Prepared profile files: $profileRoot"
    Write-Output 'REAPER was not launched. Native startup/dialog readiness remains unverified for this profile.'
    return
}
$runId = if ($ResumeRun) { $ResumeRun } else { [guid]::NewGuid().ToString('N') }
$runRoot = Join-Path $taskRoot ('.rust-validation/reaper-send/' + $runId)
if (!$ResumeRun) { New-Item -ItemType Directory -Path $runRoot -Force | Out-Null }
$instancePath = Join-Path $runRoot 'instance.json'
$project = Join-Path $runRoot 'AudioLib send proof.rpp'
$wav = Join-Path $runRoot 'teste seleção estéreo 24bit.wav'
$ready = Join-Path $runRoot 'ready.txt'
$reply = Join-Path $runRoot 'reply.txt'
$deliveredSource = Join-Path $runRoot 'source.txt'
$bootstrap = Join-Path $runRoot 'bootstrap.lua'
$send = Join-Path $runRoot 'send.lua'
$reportPath = Join-Path $runRoot 'result.json'
$requestPath = Join-Path $runRoot 'send.request'
$closePath = Join-Path $runRoot 'close.request'
$ownedProcess = $null
$passed = $false
function Lua-String([string]$Value) {
    '"' + $Value.Replace('\', '\\').Replace('"', '\"').Replace("`r", '\r').Replace("`n", '\n') + '"'
}
function Wait-Reply([string]$Path, [int]$Seconds = 40) {
    $deadline = [DateTime]::UtcNow.AddSeconds($Seconds)
    while (!(Test-Path -LiteralPath $Path)) {
        if ([DateTime]::UtcNow -gt $deadline) { throw "REAPER did not acknowledge: $Path" }
        if ($ownedProcess.HasExited) { throw 'Isolated REAPER exited before acknowledging the command.' }
        Start-Sleep -Milliseconds 100
    }
    $value = Get-Content -LiteralPath $Path -Raw
    if (!$value.StartsWith('OK')) { throw "REAPER rejected the command: $value" }
    return $value
}
if (!$ResumeRun) {
# Generate a quiet 250 ms, stereo, 48 kHz / 24-bit PCM fixture.
$frames = 12000
$bytes = $frames * 6
$writer = [IO.BinaryWriter]::new([IO.File]::Create($wav))
try {
    $writer.Write([Text.Encoding]::ASCII.GetBytes('RIFF'))
    $writer.Write([uint32](36 + $bytes))
    $writer.Write([Text.Encoding]::ASCII.GetBytes('WAVEfmt '))
    $writer.Write([uint32]16)
    $writer.Write([uint16]1); $writer.Write([uint16]2)
    $writer.Write([uint32]48000); $writer.Write([uint32]288000)
    $writer.Write([uint16]6); $writer.Write([uint16]24)
    $writer.Write([Text.Encoding]::ASCII.GetBytes('data')); $writer.Write([uint32]$bytes)
    for ($frame = 0; $frame -lt $frames; $frame++) {
        foreach ($hz in @(440, 660)) {
            $sample = [int]([Math]::Sin(2 * [Math]::PI * $hz * $frame / 48000) * 838860)
            $writer.Write([byte]($sample -band 255))
            $writer.Write([byte](($sample -shr 8) -band 255))
            $writer.Write([byte](($sample -shr 16) -band 255))
        }
    }
} finally { $writer.Dispose() }
$guard = @"
local function norm(s) return s:gsub('\\', '/'):lower() end
local function reply(path, text)
  local f = assert(io.open(path, 'wb')); f:write(text); f:close()
end
if norm(reaper.get_ini_file()) ~= norm($(Lua-String $ini)) then return end
"@
$bootBody = @"
$guard
if reaper.CountTracks(0) ~= 0 or reaper.CountMediaItems(0) ~= 0 then
  reply($(Lua-String $ready), 'ERROR: expected an empty isolated project'); return
end
reaper.InsertTrackAtIndex(0, true)
local track = reaper.GetTrack(0, 0)
reaper.GetSetMediaTrackInfo_String(track, 'P_NAME', 'AudioLib send proof', true)
reaper.SetOnlyTrackSelected(track)
reaper.SetEditCurPos(2.5, false, false)
reaper.Main_SaveProjectEx(0, $(Lua-String $project), 8)
reply($(Lua-String $ready), 'OK bridge-v1 ' .. reaper.GetAppVersion())
local consumed = false
local function exists(path)
  local f = io.open(path, 'rb'); if f then f:close(); return true end
  return false
end
local function poll()
  if exists($(Lua-String $closePath)) then
    local _, project = reaper.EnumProjects(-1, '')
    if norm(project) == norm($(Lua-String $project)) then
      reaper.Main_SaveProject(0, false)
      reaper.Main_OnCommand(40004, 0)
    end
    return
  end
  if not consumed and exists($(Lua-String $requestPath)) then
    consumed = true
    dofile($(Lua-String $send))
  end
  reaper.defer(poll)
end
reaper.defer(poll)
"@
$sendBody = @"
$guard
local ok, result = xpcall(function()
  local _, project = reaper.EnumProjects(-1, '')
  assert(norm(project) == norm($(Lua-String $project)), 'Wrong project')
  assert(reaper.CountMediaItems(0) == 0, 'Duplicate delivery')
  local track = assert(reaper.GetSelectedTrack(0, 0), 'No selected track')
  local cursor = reaper.GetCursorPosition()
  assert(math.abs(cursor - 2.5) < 0.000001, 'Unexpected cursor')
  reaper.Undo_BeginBlock2(0)
  local inserted = reaper.InsertMedia($(Lua-String $wav), 0)
  reaper.SetEditCurPos(cursor, false, false)
  reaper.Undo_EndBlock2(0, 'AudioLib: send WAV', -1)
  assert(inserted > 0 and reaper.CountMediaItems(0) == 1, 'Insertion failed')
  local item = assert(reaper.GetMediaItem(0, 0))
  assert(reaper.GetMediaItem_Track(item) == track, 'Wrong track')
  assert(math.abs(reaper.GetMediaItemInfo_Value(item, 'D_POSITION') - cursor) < 0.000001, 'Wrong position')
  assert(math.abs(reaper.GetMediaItemInfo_Value(item, 'D_LENGTH') - 0.25) < 0.000001, 'Wrong duration')
  local take = assert(reaper.GetActiveTake(item))
  local source = reaper.GetMediaItemTake_Source(take)
  -- REAPER may copy imported media into the project's Media directory. The
  -- harness validates that this delivered source is owned and byte-identical.
  local source_file = reaper.GetMediaSourceFileName(source, '')
  assert(source_file ~= '', 'Missing source file')
  reply($(Lua-String $deliveredSource), source_file)
  assert(reaper.GetMediaSourceSampleRate(source) == 48000, 'Wrong sample rate')
  assert(reaper.GetMediaSourceNumChannels(source) == 2, 'Wrong channel count')
  reaper.Main_SaveProjectEx(0, $(Lua-String $project), 8)
  return 'OK: selected track, cursor 2.5 s, duration 0.25 s, 48000 Hz, stereo'
end, debug.traceback)
reply($(Lua-String $reply), ok and result or ('ERROR: ' .. result))
"@
foreach ($entry in @(@($bootstrap, $bootBody), @($send, $sendBody))) {
    [IO.File]::WriteAllText($entry[0], $entry[1], [Text.UTF8Encoding]::new($false))
}
}
$beforeHash = (Get-FileHash -LiteralPath $wav -Algorithm SHA256).Hash
try {
    if ($ResumeRun) {
        $instance = Get-Content -LiteralPath $instancePath -Raw | ConvertFrom-Json
        if ($instance.profile_ini -ine $ini) { throw 'This run belongs to another profile or an older harness; refusing to resume.' }
        $ownedProcess = Get-Process -Id $instance.pid
        if ($ownedProcess.StartTime.ToUniversalTime().Ticks -ne $instance.start_ticks -or
            $ownedProcess.Path -ine (Get-Item -LiteralPath $ReaperPath).FullName) {
            throw 'Saved instance identity no longer matches; refusing to resume.'
        }
        if (Test-Path -LiteralPath $reply) { throw 'This run already attempted delivery; start a fresh proof.' }
    } else {
        $ownedProcess = Start-Process -FilePath $ReaperPath -ArgumentList @('-newinst', '-nosplash', '-cfgfile', ('"' + $ini + '"'), '-new', ('"' + $bootstrap + '"')) -WindowStyle Hidden -PassThru
        $instance = @{ pid = $ownedProcess.Id; start_ticks = $ownedProcess.StartTime.ToUniversalTime().Ticks; run_id = $runId; profile_ini = $ini }
        $instance |
            ConvertTo-Json | Set-Content -LiteralPath $instancePath -Encoding utf8
        $instance | ConvertTo-Json | Set-Content -LiteralPath $activeInstancePath -Encoding utf8
    }
    Write-Output "Waiting for isolated REAPER (PID $($ownedProcess.Id)); run $runId"
    Write-Output 'A fresh REAPER profile may ask for an audio device or display its evaluation notice. Insertion does not require audio playback.'
    $readyText = Wait-Reply $ready $StartupTimeoutSeconds
    if (!$readyText.StartsWith('OK bridge-v1 ')) { throw 'This older run has no local bridge; start a fresh proof.' }
    Write-Output $readyText
    # Bind delivery to this instance's owned queue rather than relying on
    # command-line routing among several open REAPER instances.
    [IO.File]::Open($requestPath, [IO.FileMode]::CreateNew).Dispose()
    Write-Output (Wait-Reply $reply)
    if ((Get-FileHash -LiteralPath $wav -Algorithm SHA256).Hash -cne $beforeHash) { throw 'Original WAV bytes changed.' }
    $actualSource = [IO.File]::ReadAllText($deliveredSource)
    if (![IO.Path]::IsPathRooted($actualSource)) { $actualSource = Join-Path $runRoot $actualSource }
    $actualSource = [IO.Path]::GetFullPath($actualSource)
    if (!$actualSource.StartsWith([IO.Path]::GetFullPath($runRoot) + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Delivered source is outside the owned proof directory.'
    }
    if ((Get-FileHash -LiteralPath $actualSource -Algorithm SHA256).Hash -cne $beforeHash) { throw 'Delivered source bytes differ from the original.' }
    $focus = 'not-requested'
    if ($TestFocus) {
        if (!('AudioLibReaperProofNative' -as [type])) {
            Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class AudioLibReaperProofNative {
    private delegate bool EnumWindowCallback(IntPtr window, IntPtr parameter);
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumWindowCallback callback, IntPtr parameter);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetWindowText(IntPtr window, StringBuilder text, int length);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window, int command);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    public static IntPtr FindOwnedWindow(uint process) {
        IntPtr result = IntPtr.Zero;
        int matches = 0;
        if (!EnumWindows((window, parameter) => {
            uint owner;
            GetWindowThreadProcessId(window, out owner);
            var title = new StringBuilder(512);
            GetWindowText(window, title, title.Capacity);
            if (owner == process && title.ToString().StartsWith("AudioLib send proof", StringComparison.Ordinal)) {
                result = window; matches++;
            }
            return true;
        }, IntPtr.Zero)) throw new InvalidOperationException("Cannot enumerate REAPER windows");
        if (matches != 1) throw new InvalidOperationException("Expected one owned REAPER project window");
        return result;
    }
}
'@
        }
        $ownedProcess.Refresh()
        $window = [AudioLibReaperProofNative]::FindOwnedWindow($ownedProcess.Id)
        [AudioLibReaperProofNative]::ShowWindow($window, 9) | Out-Null
        [AudioLibReaperProofNative]::SetForegroundWindow($window) | Out-Null
        Start-Sleep -Milliseconds 150
        $focus = if ([AudioLibReaperProofNative]::GetForegroundWindow() -eq $window) { 'passed' } else { 'denied-by-windows' }
    }
    [ordered]@{ insertion = 'passed'; original_bytes = 'unchanged'; delivered_bytes = 'identical'; sample_rate = 48000; bit_depth = 24; channels = 2; cursor_seconds = 2.5; auto_focus = $focus; project = $project; source = $wav; delivered_source = $actualSource } |
        ConvertTo-Json | Set-Content -LiteralPath $reportPath -Encoding utf8
    $passed = $true
    Write-Output "PASS insertion and source preservation; auto-focus: $focus"
    Write-Output "Report: $reportPath"
} finally {
    # Close only our saved, successfully verified instance; retain failed sessions for diagnosis.
    if ($passed -and !$KeepOpen -and $ownedProcess -and !$ownedProcess.HasExited) {
        [IO.File]::WriteAllText($closePath, 'close owned test instance')
        if (!$ownedProcess.WaitForExit(10000)) { Write-Warning "Test REAPER still open (PID $($ownedProcess.Id)); close it normally." }
    }
    Write-Output "Evidence retained in: $runRoot"
    if (!$passed) { Write-Output "In the agreed interactive test window, resume with -ResumeRun $runId -Interactive" }
}
} finally { $profileLease.Dispose() }
