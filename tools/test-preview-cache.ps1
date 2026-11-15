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
$planText=& $shell -NoProfile -File $cleaner -Root $fixture -Action Plan -Mode Debug -MaxGiB 0
if($LASTEXITCODE -ne 0){throw 'Synthetic cleanup plan failed'}
$plan=$planText | ConvertFrom-Json
if(@($plan.candidates).Count -ne 3 -or !(Test-Path -LiteralPath (Join-Path $target 'preview/sample.bin'))){throw 'Plan must include three development caches without removing them'}
$runText=& $shell -NoProfile -File $cleaner -Root $fixture -Action Run -Mode Debug -MaxGiB 0
if($LASTEXITCODE -ne 0){throw 'Synthetic cleanup run failed'}
$run=$runText | ConvertFrom-Json
if($run.status -ne 'passed' -or $run.removed_bytes -ne 96 -or $run.remaining_bytes -ne 32){throw 'Development cache cleanup did not match owned fixture contents'}
foreach($relative in @('debug','preview','x86_64-pc-windows-msvc/preview')) {
    if(Test-Path -LiteralPath (Join-Path $target $relative)){throw "Cache retained unexpectedly: $relative"}
}
if(!(Test-Path -LiteralPath (Join-Path $target 'release/sample.bin')) -or [IO.File]::ReadAllText($sentinel) -cne 'preserve'){throw 'Release or user-data sentinel changed'}
@{status='passed';checks=@('read-only plan includes debug/preview caches','run removes host and target-specific preview/debug only','release and outside-cache user data preserved');evidence=$fixture} | ConvertTo-Json
