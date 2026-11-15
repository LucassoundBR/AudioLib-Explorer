# tools/test-reaper-send-policy.ps1
# Verify that unattended invocations cannot launch REAPER or take focus.
#Requires -Version 7.3
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$harness = Join-Path $PSScriptRoot 'test-reaper-send.ps1'
$taskRoot = Split-Path -Parent $PSScriptRoot
$outputRoot = Join-Path $taskRoot '.rust-validation/reaper-send'
$profileRoot = Join-Path $taskRoot '.rust-validation/reaper-profile'
$beforeRuns = if (Test-Path -LiteralPath $outputRoot) { @(Get-ChildItem -LiteralPath $outputRoot -Directory | Select-Object -ExpandProperty Name) } else { @() }
$beforeProfile = Test-Path -LiteralPath $profileRoot
$launches = [Collections.Generic.List[string]]::new()
# Shadow process launch as an additional guard: these cases must fail before it.
function Start-Process {
    param($FilePath, $ArgumentList, $WindowStyle, [switch]$PassThru)
    $launches.Add([string]$FilePath)
    throw 'Unexpected process launch during background-policy verification.'
}
$checks = 0
foreach ($case in @(
    @{ arguments = @{ Run = $true }; expected = 'REAPER test deferred:' },
    @{ arguments = @{ Run = $true; SkipFocus = $true }; expected = 'REAPER test deferred:' },
    @{ arguments = @{ Run = $true; TestFocus = $true }; expected = 'REAPER test deferred:' },
    @{ arguments = @{ ResumeRun = ('a' * 32) }; expected = 'REAPER test deferred:' },
    @{ arguments = @{ TestFocus = $true }; expected = 'Auto-Focus testing requires' },
    @{ arguments = @{ Run = $true; Interactive = $true; TestFocus = $true; SkipFocus = $true }; expected = 'TestFocus and SkipFocus' },
    @{ arguments = @{ PrepareProfile = $true; Run = $true; Interactive = $true }; expected = 'PrepareProfile only prepares files' }
)) {
    $parameters = $case.arguments
    # Missing executable proves the desktop guard runs before dependency checks.
    $parameters.ReaperPath = Join-Path $taskRoot '.rust-validation/never-launch-reaper.exe'
    $failure = $null
    try { & $harness @parameters | Out-Null } catch { $failure = $_.Exception.Message }
    if (!$failure -or !$failure.StartsWith($case.expected)) { throw "Unexpected policy outcome: $failure" }
    $checks++
}
if ($launches.Count -ne 0) { throw 'A guarded invocation reached process launch.' }
$afterRuns = if (Test-Path -LiteralPath $outputRoot) { @(Get-ChildItem -LiteralPath $outputRoot -Directory | Select-Object -ExpandProperty Name) } else { @() }
if (@(Compare-Object $beforeRuns $afterRuns).Count -ne 0 -or (Test-Path -LiteralPath $profileRoot) -ne $beforeProfile) {
    throw 'A guarded invocation prepared native test files.'
}
Write-Output "PASS $checks desktop-policy cases; zero launches and no native-test preparation."
