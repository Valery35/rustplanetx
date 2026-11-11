# RustPlanetX - расширение RustGIS для PlanetX Lite.
# Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
# SPDX-License-Identifier: Apache-2.0
#
# Пакет .rgplugin расширения-процесса: plugin.json и программа RustPlanetX
# (PlanetX Lite для RustGIS) в bin/: для Windows bin/planetx-lite.exe, для
# Linux bin/linux/planetx-lite, для macOS bin/macos/planetx-lite (универсальная,
# Apple Silicon и Intel). Команда расширения запускает программу своей системы
# с ключом rustgis-plugin, та открывает глобус со связью с RustGIS.
#
#   pwsh -File scripts/package.ps1 -Windows rustplanetx.exe -Linux rustplanetx -Mac rustplanetx
#
# Программа без пути не кладётся, и строка её системы уходит из run пакета.
# Флаг исполнения Unix пакет получает, только если его собирает pwsh
# на Linux или macOS: каталог RustGIS проверяет его у программ Linux и macOS.
#
# В dist/ появляются:
#   rustplanetx/                  plugin.json и bin/ - папка для
#                                 «Add Plugin Folder…» и rustgis-cli --plugin
#   rustplanetx-<версия>.rgplugin пакет для выпуска
#   rustplanetx.json              запись реестра opengeos/rustgis-plugins

param(
    [Alias('Lite')][string]$Windows = '',
    [string]$Linux = '',
    [string]$Mac = ''
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root 'dist'
$manifestPath = Join-Path $root 'plugin.json'
$manifestText = [IO.File]::ReadAllText($manifestPath, [Text.Encoding]::UTF8)
$manifest = $manifestText | ConvertFrom-Json
$id = $manifest.id
$version = $manifest.version

# Программы по системам: ключ run, путь в пакете, файл на диске.
$programs = @(
    @{ platform = 'windows'; path = 'bin/planetx-lite.exe'; source = $Windows; unix = $false },
    @{ platform = 'linux'; path = 'bin/linux/planetx-lite'; source = $Linux; unix = $true },
    @{ platform = 'macos'; path = 'bin/macos/planetx-lite'; source = $Mac; unix = $true }
)
$given = @($programs | Where-Object { $_.source })
if ($given.Count -eq 0) { throw 'no program: give -Windows, -Linux or -Mac' }
foreach ($p in $given) {
    if (-not (Test-Path $p.source)) { throw "no program for $($p.platform) at $($p.source)" }
    $line = '"' + $p.platform + '": ["' + $p.path + '"'
    if (-not $manifestText.Contains($line)) { throw "plugin.json run has no $line" }
}

# run пакета - только системы с программой: строка системы без программы
# уходит, запятая после последней оставшейся строки - тоже.
foreach ($p in @($programs | Where-Object { -not $_.source })) {
    $manifestText = [regex]::Replace($manifestText, '(?m)^[ \t]*"' + $p.platform + '": \[[^\]]*\],?\r?\n', '')
}
$manifestText = [regex]::Replace($manifestText, '(\])\s*,(\s*\r?\n[ \t]*\},\s*\r?\n[ \t]*"minRustgisVersion")', '$1$2')
$null = $manifestText | ConvertFrom-Json
$manifestBytes = (New-Object Text.UTF8Encoding($false)).GetBytes($manifestText)

New-Item -ItemType Directory -Force $dist | Out-Null
$folder = Join-Path $dist $id
if (Test-Path $folder) { Remove-Item -Recurse -Force $folder -Confirm:$false }
New-Item -ItemType Directory -Force $folder | Out-Null

# Файлы пакета: путь в пакете, байты, исполняемый ли под Unix.
$files = @(@{ path = 'plugin.json'; bytes = $manifestBytes; unix = $false })
foreach ($p in $given) {
    $files += @{ path = $p.path; bytes = [IO.File]::ReadAllBytes((Resolve-Path $p.source).Path); unix = $p.unix }
}
# Навыки помощника RustGIS: skills/<имя>/SKILL.md из plugin.json.
foreach ($skill in @($manifest.skills)) {
    if (-not $skill) { continue }
    $source = [IO.Path]::Combine($root, 'skills', $skill, 'SKILL.md')
    if (-not (Test-Path $source)) { throw "no skill $skill at $source" }
    $files += @{ path = "skills/$skill/SKILL.md"; bytes = [IO.File]::ReadAllBytes($source); unix = $false }
}

# Папка для «Add Plugin Folder…».
foreach ($f in $files) {
    $target = $folder
    foreach ($part in $f.path.Split('/')) { $target = Join-Path $target $part }
    New-Item -ItemType Directory -Force (Split-Path -Parent $target) | Out-Null
    [IO.File]::WriteAllBytes($target, $f.bytes)
    if ($f.unix -and -not $IsWindows -and $null -ne $IsWindows) { chmod 755 $target }
}

# Пакет - ZIP с файлами в корне. Время записей одно и то же, тогда
# одинаковые файлы дают одинаковый пакет и SHA-256. Режим файлов Unix:
# программы Linux и macOS - 0755, остальное - 0644. Он действует, когда
# пакет собирает pwsh на Linux или macOS: запись тогда сделана «в Unix».
Add-Type -AssemblyName System.IO.Compression
# Режим - в старших 16 битах ExternalAttributes (Int32): 0x81ED0000 и 0x81A40000.
$modeExec = [int]-2115174400
$modeFile = [int]-2119958528
$package = Join-Path $dist "$id-$version.rgplugin"
if (Test-Path $package) { Remove-Item $package -Confirm:$false }
$stream = [System.IO.File]::Open($package, [System.IO.FileMode]::CreateNew)
try {
    $zip = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Create)
    try {
        foreach ($f in $files) {
            $entry = $zip.CreateEntry($f.path, [System.IO.Compression.CompressionLevel]::Optimal)
            $entry.LastWriteTime = [DateTimeOffset]::new(2024, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
            $entry.ExternalAttributes = if ($f.unix) { $modeExec } else { $modeFile }
            $out = $entry.Open()
            try { $out.Write($f.bytes, 0, $f.bytes.Length) } finally { $out.Dispose() }
        }
    } finally { $zip.Dispose() }
} finally { $stream.Dispose() }

# Каталог RustGIS не берёт пакет больше 64 МБ.
$size = (Get-Item $package).Length
if ($size -gt 64MB) { throw ("package is {0:N0} bytes, over the 64 MB of the RustGIS registry" -f $size) }

$sha = (Get-FileHash $package -Algorithm SHA256).Hash.ToLowerInvariant()
$url = "https://github.com/Valery35/rustplanetx/releases/download/v$version/$id-$version.rgplugin"
$entryJson = [ordered]@{ id = $id; source = [ordered]@{ url = $url; sha256 = $sha }; categories = @('Visualization', 'Terrain') } | ConvertTo-Json -Depth 3
[System.IO.File]::WriteAllText((Join-Path $dist "$id.json"), $entryJson + "`n", (New-Object System.Text.UTF8Encoding($false)))

$platforms = ($given | ForEach-Object { $_.platform }) -join ', '
Write-Output ("dist/{0}-{1}.rgplugin  {2:N0} bytes  {3}  sha256 {4}" -f $id, $version, $size, $platforms, $sha)
