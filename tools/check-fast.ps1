# tools/check-fast.ps1
# Compile/test actual protocol source without Slint, GUI, playback or live REAPER.
#Requires -Version 7.3
[CmdletBinding()]
param([ValidateSet('Plan','Run')][string]$Action='Plan')
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$manifest=Join-Path $taskRoot 'tools/fast-tests/Cargo.toml'
$target=Join-Path $taskRoot 'target'
$common=@('--manifest-path',$manifest,'--locked','--target-dir',$target,'--test','reaper-contracts')
if($Action -eq 'Plan') {
    @{action='Plan';manifest=$manifest;target=$target;scope='production-source REAPER tests; no Slint, GUI, audio or live REAPER';steps=@('strict Clippy','isolated tests');requires='configured Cargo/Rustup environment'} | ConvertTo-Json
    return
}
. (Join-Path $PSScriptRoot 'build-cache.ps1')
$lease=$null
$runRoot=Join-Path $taskRoot ('.rust-validation/fast-checks/'+[guid]::NewGuid().ToString('N'))
$records=@()
Push-Location -LiteralPath $taskRoot
try {
    $lease=Enter-AudioLibBuildLease $taskRoot
    New-Item -ItemType Directory -Force -Path $runRoot | Out-Null
    foreach($step in @('clippy','test')) {
        $arguments=@($step)+$common
        if($step -eq 'clippy'){$arguments+=@('--','-D','warnings')}
        $log=Join-Path $runRoot ($step+'.log')
        $watch=[Diagnostics.Stopwatch]::StartNew()
        & cargo @arguments *> $log
        $code=$LASTEXITCODE
        $watch.Stop()
        $records+=@{step=$step;seconds=$watch.Elapsed.TotalSeconds;exit_code=$code;log=$log}
        if($code -ne 0){Get-Content -LiteralPath $log -Tail 20;throw "$step failed; inspect $log"}
        if($step -eq 'test') {
            $summary=Select-String -LiteralPath $log -Pattern '^test result: ok\. (\d+) passed;' | Select-Object -Last 1
            if(!$summary -or [int]$summary.Matches[0].Groups[1].Value -eq 0){throw 'No passed tests; isolated check is not accepted'}
            $passed=[int]$summary.Matches[0].Groups[1].Value
        }
    }
    $result=@{status='passed';scope='REAPER native pure logic, focus parser, synthetic companion protocol; no Controller/UI/live host acceptance';passed=$passed;steps=$records;evidence=$runRoot}
    $result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $runRoot 'report.json')
    $result | ConvertTo-Json -Depth 5
}finally{
    if($lease){$lease.Dispose()}
    Pop-Location
}
