param(
  [Parameter(Mandatory)][ValidateSet('x64','ARM64')][string]$Architecture,
  [Parameter(Mandatory)][string]$Version,
  [Parameter(Mandatory)][ValidateSet('stable','beta','alpha')][string]$Channel,
  [Parameter(Mandatory)][string]$OutputDirectory
)
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($env:WINDOWS_UPDATE_PUBLIC_KEY)) { throw 'WINDOWS_UPDATE_PUBLIC_KEY is required.' }
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$Arch = $Architecture.ToLowerInvariant()
$Target = if ($Arch -eq 'arm64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
& rustup target add $Target
if ($LASTEXITCODE -ne 0) { throw 'Rust target setup failed.' }
$Publish = Join-Path $Repo "target/publish/win-$Arch-$([guid]::NewGuid().ToString('N'))"
$Tools = Join-Path $Repo 'target/release-tools'
if ((Test-Path $OutputDirectory) -and @(Get-ChildItem -Force $OutputDirectory).Count -ne 0) { throw 'Release output must be empty.' }
& dotnet publish (Join-Path $Repo 'windows/SerialStudio/SerialStudio.csproj') -c Release "-p:Platform=$Architecture" "-p:Version=$Version" -o $Publish
if ($LASTEXITCODE -ne 0) { throw 'WinUI publish failed.' }
$Notes = Join-Path $Repo "target/notes-$Arch"
& cargo run --locked --manifest-path (Join-Path $Repo 'Cargo.toml') -p xtask -- release-notes (Join-Path $Repo "release-notes/$Version") $Notes
if ($LASTEXITCODE -ne 0) { throw 'Platform announcement generation failed.' }
if (-not (Test-Path (Join-Path $Tools 'vpk.exe'))) {
  & dotnet tool install vpk --version 1.2.161 --tool-path $Tools
  if ($LASTEXITCODE -ne 0) { throw 'Velopack installation failed.' }
}
& (Join-Path $Tools 'vpk.exe') pack --packId SerialStudio --packVersion $Version --packDir $Publish --mainExe SerialStudio.exe --runtime "win10.0.19041-$Arch" --framework "vcredist143-$Arch" --channel "win-$Arch-$Channel" --outputDir $OutputDirectory --icon (Join-Path $Repo 'assets/icon.ico') --releaseNotes (Join-Path $Notes 'notes-windows.md')
if ($LASTEXITCODE -ne 0) { throw 'Velopack packaging failed.' }
$Installers = @(Get-ChildItem $OutputDirectory -File -Filter '*Setup.exe')
if ($Installers.Count -ne 1) { throw 'Expected one Velopack installer.' }
Move-Item $Installers[0].FullName (Join-Path $OutputDirectory "SerialStudio-$Version-windows-$Arch.exe")
Get-ChildItem $OutputDirectory -File | Where-Object { $_.Extension -eq '.zip' -or ($_.Name -like 'RELEASES*' -and $_.Extension -ne '.json') } | Remove-Item
$FeedPath = Join-Path $OutputDirectory "releases.win-$Arch-$Channel.json"
$Feed = Get-Content -Raw $FeedPath | ConvertFrom-Json
$Feed.Assets = @($Feed.Assets | Where-Object { $_.Type -eq 'Full' -or $_.Type -eq 1 })
if ($Feed.Assets.Count -ne 1) { throw 'Expected one full update package.' }
$Feed | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $FeedPath
