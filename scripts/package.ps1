# RustPlanetX - расширение RustGIS для PlanetX Lite.
# Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
# SPDX-License-Identifier: Apache-2.0
#
# Сборка модуля и пакета .rgplugin, как scripts/package.py шаблона
# opengeos/rustgis-plugin-template (MIT), без Python:
#
#   powershell -File scripts/package.ps1            # сборка и пакет в dist/
#   powershell -File scripts/package.ps1 -NoBuild   # пакет из уже собранного модуля
#
# В dist/ появляются:
#   rustplanetx/                  plugin.json и plugin.wasm - папка для
#                                 «Add Plugin Folder…» и rustgis-cli --plugin
#   rustplanetx-<версия>.rgplugin пакет для выпуска
#   rustplanetx.json              запись реестра opengeos/rustgis-plugins

param([switch]$NoBuild)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root 'dist'
$manifestPath = Join-Path $root 'plugin.json'
$manifest = Get-Content $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
$id = $manifest.id
$version = $manifest.version

if (-not $NoBuild) {
    Push-Location $root
    try {
        cargo build --release --target wasm32-unknown-unknown
        if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
    } finally { Pop-Location }
}
# Путь по частям: скрипт идёт и в CI на Linux.
$wasm = [System.IO.Path]::Combine($root, 'target', 'wasm32-unknown-unknown', 'release', 'rustplanetx.wasm')
if (-not (Test-Path $wasm)) { throw "no module at $wasm" }

New-Item -ItemType Directory -Force $dist | Out-Null
$folder = Join-Path $dist $id
New-Item -ItemType Directory -Force $folder | Out-Null
Copy-Item $manifestPath (Join-Path $folder 'plugin.json') -Force
Copy-Item $wasm (Join-Path $folder 'plugin.wasm') -Force

# Пакет - ZIP с plugin.json и модулем в корне. Время записей одно
# и то же, тогда одинаковые файлы дают одинаковый пакет и SHA-256.
Add-Type -AssemblyName System.IO.Compression
$package = Join-Path $dist "$id-$version.rgplugin"
if (Test-Path $package) { Remove-Item $package -Confirm:$false }
$stream = [System.IO.File]::Open($package, [System.IO.FileMode]::CreateNew)
try {
    $zip = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Create)
    try {
        foreach ($pair in @(@('plugin.json', $manifestPath), @('plugin.wasm', $wasm))) {
            $entry = $zip.CreateEntry($pair[0], [System.IO.Compression.CompressionLevel]::Optimal)
            $entry.LastWriteTime = [DateTimeOffset]::new(2024, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
            $out = $entry.Open()
            try {
                $bytes = [System.IO.File]::ReadAllBytes($pair[1])
                $out.Write($bytes, 0, $bytes.Length)
            } finally { $out.Dispose() }
        }
    } finally { $zip.Dispose() }
} finally { $stream.Dispose() }

$sha = (Get-FileHash $package -Algorithm SHA256).Hash.ToLowerInvariant()
$url = "https://github.com/Valery35/rustplanetx/releases/download/v$version/$id-$version.rgplugin"
$entryJson = [ordered]@{ id = $id; source = [ordered]@{ url = $url; sha256 = $sha }; categories = @('Basemaps', 'Imagery') } | ConvertTo-Json -Depth 3
[System.IO.File]::WriteAllText((Join-Path $dist "$id.json"), $entryJson + "`n", (New-Object System.Text.UTF8Encoding($false)))

Write-Output ("dist/{0}-{1}.rgplugin  {2:N0} bytes  sha256 {3}" -f $id, $version, (Get-Item $package).Length, $sha)
