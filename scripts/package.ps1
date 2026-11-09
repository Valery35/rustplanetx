# RustPlanetX - расширение RustGIS для PlanetX Lite.
# Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
# SPDX-License-Identifier: Apache-2.0
#
# Пакет .rgplugin расширения-процесса: plugin.json и программа PlanetX Lite
# для Windows в bin/: голое имя программы RustGIS ищет в PATH, путь - в
# папке расширения. Команда расширения запускает `bin/planetx-lite.exe
# rustgis-plugin`, та открывает глобус со связью с RustGIS.
#
#   powershell -File scripts/package.ps1 -Lite путь\planetx-lite.exe
#
# В dist/ появляются:
#   rustplanetx/                  plugin.json и bin/planetx-lite.exe - папка для
#                                 «Add Plugin Folder…» и rustgis-cli --plugin
#   rustplanetx-<версия>.rgplugin пакет для выпуска
#   rustplanetx.json              запись реестра opengeos/rustgis-plugins

param([Parameter(Mandatory = $true)][string]$Lite)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root 'dist'
$manifestPath = Join-Path $root 'plugin.json'
$manifest = Get-Content $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
$id = $manifest.id
$version = $manifest.version
if (-not (Test-Path $Lite)) { throw "no PlanetX Lite at $Lite" }

New-Item -ItemType Directory -Force $dist | Out-Null
$folder = Join-Path $dist $id
if (Test-Path $folder) { Remove-Item -Recurse -Force $folder -Confirm:$false }
New-Item -ItemType Directory -Force $folder | Out-Null
Copy-Item $manifestPath (Join-Path $folder 'plugin.json') -Force
New-Item -ItemType Directory -Force (Join-Path $folder 'bin') | Out-Null
Copy-Item $Lite (Join-Path $folder 'bin\planetx-lite.exe') -Force

# Навыки помощника RustGIS: skills/<имя>/SKILL.md из plugin.json.
$files = @(@('plugin.json', $manifestPath), @('bin/planetx-lite.exe', $Lite))
foreach ($skill in @($manifest.skills)) {
    if (-not $skill) { continue }
    $source = Join-Path $root "skills\$skill\SKILL.md"
    if (-not (Test-Path $source)) { throw "no skill $skill at $source" }
    New-Item -ItemType Directory -Force (Join-Path $folder "skills\$skill") | Out-Null
    Copy-Item $source (Join-Path $folder "skills\$skill\SKILL.md") -Force
    $files += , @("skills/$skill/SKILL.md", $source)
}

# Пакет - ZIP с файлами в корне. Время записей одно и то же, тогда
# одинаковые файлы дают одинаковый пакет и SHA-256.
Add-Type -AssemblyName System.IO.Compression
$package = Join-Path $dist "$id-$version.rgplugin"
if (Test-Path $package) { Remove-Item $package -Confirm:$false }
$stream = [System.IO.File]::Open($package, [System.IO.FileMode]::CreateNew)
try {
    $zip = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Create)
    try {
        foreach ($pair in $files) {
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
$entryJson = [ordered]@{ id = $id; source = [ordered]@{ url = $url; sha256 = $sha }; categories = @('Visualization', 'Terrain') } | ConvertTo-Json -Depth 3
[System.IO.File]::WriteAllText((Join-Path $dist "$id.json"), $entryJson + "`n", (New-Object System.Text.UTF8Encoding($false)))

Write-Output ("dist/{0}-{1}.rgplugin  {2:N0} bytes  sha256 {3}" -f $id, $version, (Get-Item $package).Length, $sha)
