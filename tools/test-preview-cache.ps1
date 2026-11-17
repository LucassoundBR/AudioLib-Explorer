# tools/test-preview-cache.ps1
# Functional cleanup boundary regression, exclusively in an owned synthetic tree.
#Requires -Version 7.3
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
$parent=[IO.Path]::GetFullPath((Join-Path $root '.rust-validation/preview-cache-tests'))
$fixture=[IO.Path]::GetFullPath((Join-Path $parent ([guid]::NewGuid().ToString('N'))))
if(!$fixture.StartsWith($parent+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Fixture escaped owned subtree'}
$target=Join-Path $fixture 'target'
New-Item -ItemType Directory -Force -Path $target | Out-Null
[IO.File]::WriteAllText((Join-Path $fixture 'Cargo.toml'),'# owned fixture: crates/audiolib-core')
& git -C $fixture init -q
if($LASTEXITCODE -ne 0){throw 'Isolated fixture Git initialization failed'}
[IO.File]::WriteAllText((Join-Path $target 'CACHEDIR.TAG'),'Signature: 8a477f597d28d172789f06886806bc55')
foreach($relative in @('debug','preview','x86_64-pc-windows-msvc/preview','release')) {
    $path=Join-Path $target $relative
    New-Item -ItemType Directory -Force -Path $path | Out-Null
    [IO.File]::WriteAllBytes((Join-Path $path 'sample.bin'),[byte[]]::new(32))
}
$sentinel=Join-Path $fixture 'preserve-user-data.bin'
[IO.File]::WriteAllText($sentinel,'preserve')
$shell=(Get-Process -Id $PID).Path
$cleaner=Join-Path $PSScriptRoot 'clean-build-artifacts.ps1'
$planText=& $shell -NoProfile -File $cleaner -Root $fixture -TargetDirectory $target -Action Plan -Mode Debug -MaxGiB 0
if($LASTEXITCODE -ne 0){throw 'Synthetic cleanup plan failed'}
$plan=$planText | ConvertFrom-Json
if(@($plan.candidates).Count -ne 3 -or !(Test-Path -LiteralPath (Join-Path $target 'preview/sample.bin'))){throw 'Plan must include three development caches without removing them'}
$runText=& $shell -NoProfile -File $cleaner -Root $fixture -TargetDirectory $target -Action Run -Mode Debug -MaxGiB 0
if($LASTEXITCODE -ne 0){throw 'Synthetic cleanup run failed'}
$run=$runText | ConvertFrom-Json
if($run.status -ne 'passed' -or $run.removed_bytes -ne 96 -or $run.remaining_bytes -ne 32){throw 'Development cache cleanup did not match owned fixture contents'}
foreach($relative in @('debug','preview','x86_64-pc-windows-msvc/preview')) {
    if(Test-Path -LiteralPath (Join-Path $target $relative)){throw "Cache retained unexpectedly: $relative"}
}
if(!(Test-Path -LiteralPath (Join-Path $target 'release/sample.bin')) -or [IO.File]::ReadAllText($sentinel) -cne 'preserve'){throw 'Release or user-data sentinel changed'}
function New-PreviewIntermediates([string]$Destination,[switch]$CargoEvidence) {
    foreach($relative in @('preview','x86_64-pc-windows-msvc/preview')) {
        $profile=Join-Path $Destination $relative
        foreach($name in @('build','deps','.fingerprint','incremental','examples')) {
            $path=Join-Path $profile $name
            New-Item -ItemType Directory -Force -Path $path | Out-Null
            [IO.File]::WriteAllBytes((Join-Path $path 'sample.bin'),[byte[]]::new(32))
        }
        foreach($name in @('app.exe','app.pdb')) {
            [IO.File]::WriteAllBytes((Join-Path $profile $name),[byte[]]::new(16))
        }
        if($CargoEvidence) {
            New-Item -ItemType Directory -Path (Join-Path $profile '.fingerprint/owned_fixture-0123456789abcdef') | Out-Null
            [IO.File]::WriteAllBytes((Join-Path $profile 'deps/libowned_fixture-0123456789abcdef.rmeta'),[byte[]]::new(32))
        }
    }
}
New-PreviewIntermediates $target
foreach($relative in @('tmp','x86_64-pc-windows-msvc/tmp','preview/keep')) {
    $path=Join-Path $target $relative
    New-Item -ItemType Directory -Force -Path $path | Out-Null
    [IO.File]::WriteAllBytes((Join-Path $path 'sample.bin'),[byte[]]::new(32))
}
$intermediatePlan=& $shell -NoProfile -File $cleaner -Root $fixture -TargetDirectory $target -Action Plan -Mode Intermediate -MaxGiB 0
if($LASTEXITCODE -ne 0){throw 'Intermediate preview plan failed'}
$intermediatePlan=$intermediatePlan | ConvertFrom-Json
if(@($intermediatePlan.candidates).Count -ne 12 -or $intermediatePlan.removed_bytes -ne 0 -or !(Test-Path -LiteralPath (Join-Path $target 'preview/deps/sample.bin'))){throw 'Intermediate plan must inventory both preview profiles and tmp without deletion'}
$intermediateRun=& $shell -NoProfile -File $cleaner -Root $fixture -TargetDirectory $target -Action Run -Mode Intermediate -MaxGiB 0
if($LASTEXITCODE -ne 0){throw 'Intermediate preview run failed'}
$intermediateRun=$intermediateRun | ConvertFrom-Json
if($intermediateRun.status -ne 'passed' -or $intermediateRun.removed_bytes -ne 384 -or $intermediateRun.remaining_bytes -ne 128){throw 'Intermediate cleanup must remove only generated preview subdirectories and tmp'}
foreach($relative in @('preview','x86_64-pc-windows-msvc/preview')) {
    foreach($name in @('build','deps','.fingerprint','incremental','examples')) {
        if(Test-Path -LiteralPath (Join-Path $target ($relative+'/'+$name))){throw "Preview intermediate retained: $relative/$name"}
    }
    foreach($name in @('app.exe','app.pdb')) {
        if((Get-Item -LiteralPath (Join-Path $target ($relative+'/'+$name))).Length -ne 16){throw 'Intermediate cleanup changed final preview outputs'}
    }
}
if(!(Test-Path -LiteralPath (Join-Path $target 'release/sample.bin')) -or !(Test-Path -LiteralPath (Join-Path $target 'preview/keep/sample.bin')) -or [IO.File]::ReadAllText($sentinel) -cne 'preserve'){throw 'Intermediate cleanup exceeded the owned subdirectory boundary'}

# External untagged caches must recognize actual preview compiler artifacts and
# retain only their narrowly scoped Intermediate resume proof after pruning.
$registered=Join-Path $fixture 'registered-preview-target'
$null=& $shell -NoProfile -File $cleaner -Root $fixture -TargetDirectory $registered -Action Register
if($LASTEXITCODE -ne 0){throw 'External preview registration failed'}
New-PreviewIntermediates $registered -CargoEvidence
$compilerState=@{rustc_fingerprint=[ulong]123456789;outputs=@{'123456789'=@{success=$true;code=0;stdout="rustc 1.97.1 (ownedfixture)`nbinary: rustc`nhost: x86_64-pc-windows-msvc`nrelease: 1.97.1`n"}}}
$compilerState | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $registered '.rustc_info.json') -Encoding utf8
foreach($iteration in 1..2) {
    $externalRun=& $shell -NoProfile -File $cleaner -Root $fixture -TargetDirectory $registered -Action Run -Mode Intermediate -MaxGiB 0
    if($LASTEXITCODE -ne 0){throw 'Registered preview Intermediate cleanup/resume failed'}
    $externalRun=$externalRun | ConvertFrom-Json
    $expected=if($iteration -eq 1){384}else{0}
    if($externalRun.removed_bytes -ne $expected -or $externalRun.remaining_bytes -ne 64 -or $externalRun.ownership -ne 'registered'){throw 'Registered preview cleanup lost its ownership or retained-output proof'}
}
$null=& $shell -NoProfile -File $cleaner -Root $fixture -TargetDirectory $registered -Action Run -Mode All -MaxGiB 0 2>&1
if($LASTEXITCODE -eq 0 -or !(Test-Path -LiteralPath (Join-Path $registered 'preview/app.exe'))){throw 'Intermediate proof must not authorize whole-profile deletion'}
@{status='passed';checks=@('explicit fixture target avoids managed Windows cache routing','read-only Debug/Intermediate plans include preview caches','Debug removes development profiles and preserves release/user data','Intermediate removes host/triple preview intermediates and tmp, retaining final outputs and unknown data','registered untagged preview evidence supports repeat Intermediate cleanup but never All');evidence=$fixture} | ConvertTo-Json
