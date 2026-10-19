param([ValidateSet('x64', 'ARM64')][string]$Architecture = 'x64', [ValidateSet('Debug','Release')][string]$Configuration = 'Debug')
$ErrorActionPreference = 'Stop'
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$Target = if ($Architecture -eq 'ARM64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
& rustup target add $Target
if ($LASTEXITCODE -ne 0) { throw 'Rust target setup failed.' }
& dotnet build (Join-Path $Repo 'windows/SerialStudio/SerialStudio.csproj') -c $Configuration "-p:Platform=$Architecture"
if ($LASTEXITCODE -ne 0) { throw 'WinUI build failed.' }
