param([switch]$DebugBuild, [string]$CompilerBin)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
if (-not $CompilerBin) {
    $localCompiler = Get-ChildItem '.local-tools\native\llvm-mingw-*\bin\x86_64-w64-mingw32-clang.exe' -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($localCompiler) { $CompilerBin = $localCompiler.DirectoryName }
}
if ($CompilerBin) { $env:PATH = "$CompilerBin;$env:PATH" }
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    $candidate = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (-not (Test-Path $candidate)) { throw 'Installer Rust Windows depuis https://rustup.rs avec les outils MSVC, ou Rust GNU avec LLVM-MinGW.' }
    $cargoPath = $candidate
} else { $cargoPath = $cargo.Source }
$rustcPath = Join-Path (Split-Path $cargoPath -Parent) 'rustc.exe'
if (Test-Path $rustcPath) {
    $hostInfo = & $rustcPath -vV
    if ($hostInfo -match 'host: x86_64-pc-windows-gnu') {
        $sysroot = & $rustcPath --print sysroot
        $bundledLinker = Join-Path $sysroot 'lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained\x86_64-w64-mingw32-gcc.exe'
        if ((Test-Path $bundledLinker) -and -not $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER) { $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = $bundledLinker }
    }
}
$env:CARGO_BUILD_JOBS = '4' 
$argsBuild = @('build', '--locked')
if (-not $DebugBuild) { $argsBuild += '--release' }
# LLD accepte les bibliothèques d'import GNU et LLVM. Garder le CRT fourni par Rust.
if ($hostInfo -match 'host: x86_64-pc-windows-gnu') {
    $rustLld = Join-Path $sysroot 'lib\rustlib\x86_64-pc-windows-gnu\bin\rust-lld.exe'
    if (-not (Test-Path $rustLld)) { throw 'Le linker LLD fourni avec Rust est absent.' }
    $argsBuild = @('rustc', '--locked', '--bin', 'bouftourust')
    if (-not $DebugBuild) { $argsBuild += '--release' }
    $argsBuild += @('--', '-C', "linker=$rustLld", '-C', 'linker-flavor=ld.lld', '-C', 'link-self-contained=yes')
}
& $cargoPath @argsBuild
if ($LASTEXITCODE -ne 0) { throw 'La compilation Windows a échoué.' }
$targetRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
$profileName = if ($DebugBuild) { 'debug' } else { 'release' }
$bundle = 'dist\BouftouRush-Windows'
New-Item -ItemType Directory -Force $bundle | Out-Null
Copy-Item "$targetRoot\$profileName\bouftourust.exe" "$bundle\BouftouRush.exe" -Force
New-Item -ItemType Directory -Force "$bundle\assets" | Out-Null
Copy-Item assets\* "$bundle\assets" -Recurse -Force
Copy-Item README.md "$bundle\LISEZ-MOI.md" -Force
Copy-Item docs\ASSETS.md "$bundle\ASSETS.md" -Force
New-Item -ItemType Directory -Force "$bundle\docs" | Out-Null
Copy-Item docs\EQUILIBRAGE.md "$bundle\docs\EQUILIBRAGE.md" -Force
Write-Host "Jeu Windows prêt : $bundle\BouftouRush.exe"
