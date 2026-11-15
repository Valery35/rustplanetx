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
#   rustplanetx-<версия>.rgplugin общий пакет со всеми программами (Dropbox,
#                                 ручная установка, RustGIS до пакетов по системам)
#   rustplanetx-<версия>-<система>.rgplugin  пакет одной системы: тот же
#                                 plugin.json, в run только её строка
#   rustplanetx.json              запись реестра opengeos/rustgis-plugins
#
# -PerPlatform: запись реестра называет пакет каждой системы
# ("source": {"windows": {...}, "linux": {...}, "macos": {...}}), RustGIS
# качает только свой (opengeos/RustGIS#1821, коммит ae748ba). RustGIS старше
# этой возможности такую запись пропускает. Без ключа запись называет общий пакет.

param(
    [Alias('Lite')][string]$Windows = '',
    [string]$Linux = '',
    [string]$Mac = '',
    [switch]$PerPlatform,
    # Запись реестра называет пакет одной системы (windows, linux или macos):
    # предпросмотр заявки в каталог держит пакеты до 25 МиБ, общий пакет
    # больше, а запись с пакетами по системам RustGIS 0.6.0 пропускает.
    [ValidateSet('', 'windows', 'linux', 'macos')][string]$CatalogPlatform = ''
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

# plugin.json пакета: в run только системы из $keep. Строка системы без
# программы уходит, запятая после последней оставшейся строки - тоже.
function Get-Manifest([string[]]$keep) {
    $text = $manifestText
    foreach ($p in @($programs | Where-Object { $keep -notcontains $_.platform })) {
        $text = [regex]::Replace($text, '(?m)^[ \t]*"' + $p.platform + '": \[[^\]]*\],?\r?\n', '')
    }
    $text = [regex]::Replace($text, '(\])\s*,(\s*\r?\n[ \t]*\},\s*\r?\n[ \t]*"minRustgisVersion")', '$1$2')
    $null = $text | ConvertFrom-Json
    return , (New-Object Text.UTF8Encoding($false)).GetBytes($text)
}

# Навыки помощника RustGIS: skills/<имя>/SKILL.md из plugin.json.
$skills = @()
foreach ($skill in @($manifest.skills)) {
    if (-not $skill) { continue }
    $source = [IO.Path]::Combine($root, 'skills', $skill, 'SKILL.md')
    if (-not (Test-Path $source)) { throw "no skill $skill at $source" }
    $skills += @{ path = "skills/$skill/SKILL.md"; bytes = [IO.File]::ReadAllBytes($source); unix = $false }
}
foreach ($p in $given) { $p.bytes = [IO.File]::ReadAllBytes((Resolve-Path $p.source).Path) }

# Файлы пакета: путь в пакете, байты, исполняемый ли под Unix.
function Get-Files([object[]]$progs) {
    $list = @(@{ path = 'plugin.json'; bytes = (Get-Manifest @($progs | ForEach-Object { $_.platform })); unix = $false })
    foreach ($p in $progs) { $list += @{ path = $p.path; bytes = $p.bytes; unix = $p.unix } }
    return $list + $skills
}

# Пакет - ZIP с файлами в корне. Время записей одно и то же, тогда
# одинаковые файлы дают одинаковый пакет и SHA-256. Режим файлов Unix:
# программы Linux и macOS - 0755, остальное - 0644. Он действует, когда
# пакет собирает pwsh на Linux или macOS: запись тогда сделана «в Unix».
Add-Type -AssemblyName System.IO.Compression
# Режим - в старших 16 битах ExternalAttributes (Int32): 0x81ED0000 и 0x81A40000.
$modeExec = [int]-2115174400
$modeFile = [int]-2119958528
function Write-Package([string]$path, [object[]]$list) {
    if (Test-Path $path) { Remove-Item $path -Confirm:$false }
    $stream = [System.IO.File]::Open($path, [System.IO.FileMode]::CreateNew)
    try {
        $zip = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Create)
        try {
            foreach ($f in $list) {
                $entry = $zip.CreateEntry($f.path, [System.IO.Compression.CompressionLevel]::Optimal)
                $entry.LastWriteTime = [DateTimeOffset]::new(2024, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
                $entry.ExternalAttributes = if ($f.unix) { $modeExec } else { $modeFile }
                $out = $entry.Open()
                try { $out.Write($f.bytes, 0, $f.bytes.Length) } finally { $out.Dispose() }
            }
        } finally { $zip.Dispose() }
    } finally { $stream.Dispose() }
    # Каталог RustGIS не берёт пакет больше 64 МиБ.
    $size = (Get-Item $path).Length
    if ($size -gt 64MB) { throw ("{0} is {1:N0} bytes, over the 64 MiB of the RustGIS registry" -f (Split-Path -Leaf $path), $size) }
    return [ordered]@{ name = (Split-Path -Leaf $path); size = $size; sha256 = (Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant() }
}

New-Item -ItemType Directory -Force $dist | Out-Null
$folder = Join-Path $dist $id
if (Test-Path $folder) { Remove-Item -Recurse -Force $folder -Confirm:$false }
New-Item -ItemType Directory -Force $folder | Out-Null
foreach ($old in @(Get-ChildItem $dist -Filter "$id-*.rgplugin")) { Remove-Item $old.FullName -Confirm:$false }

# Папка для «Add Plugin Folder…»: все программы.
$files = Get-Files $given
foreach ($f in $files) {
    $target = $folder
    foreach ($part in $f.path.Split('/')) { $target = Join-Path $target $part }
    New-Item -ItemType Directory -Force (Split-Path -Parent $target) | Out-Null
    [IO.File]::WriteAllBytes($target, $f.bytes)
    if ($f.unix -and -not $IsWindows -and $null -ne $IsWindows) { chmod 755 $target }
}

$base = "https://github.com/Valery35/rustplanetx/releases/download/v$version"
$all = Write-Package (Join-Path $dist "$id-$version.rgplugin") $files
$platforms = ($given | ForEach-Object { $_.platform }) -join ', '
Write-Output ("dist/{0}  {1:N0} bytes  {2}  sha256 {3}" -f $all.name, $all.size, $platforms, $all.sha256)

# Пакет каждой системы. Предпросмотр заявки в каталог держит пакеты до 25 МиБ.
$source = [ordered]@{}
foreach ($p in $given) {
    $one = Write-Package (Join-Path $dist "$id-$version-$($p.platform).rgplugin") (Get-Files @($p))
    $note = if ($one.size -gt 25MB) { '  over the 25 MiB of the registry preview' } else { '' }
    Write-Output ("dist/{0}  {1:N0} bytes  sha256 {2}{3}" -f $one.name, $one.size, $one.sha256, $note)
    $source[$p.platform] = [ordered]@{ url = "$base/$($one.name)"; sha256 = $one.sha256 }
}
if ($PerPlatform -and $CatalogPlatform) { throw '-PerPlatform and -CatalogPlatform: give one' }
$what = "a package per platform ($platforms)"
if ($CatalogPlatform) {
    if (-not $source.Contains($CatalogPlatform)) { throw "no program for $CatalogPlatform, the catalog platform" }
    $source = $source[$CatalogPlatform]
    $what = "the $CatalogPlatform package"
} elseif (-not $PerPlatform) {
    $source = [ordered]@{ url = "$base/$($all.name)"; sha256 = $all.sha256 }
    $what = 'the package with every program'
}

$entryJson = [ordered]@{ id = $id; source = $source; categories = @('Visualization', 'Terrain') } | ConvertTo-Json -Depth 4
[System.IO.File]::WriteAllText((Join-Path $dist "$id.json"), $entryJson + "`n", (New-Object System.Text.UTF8Encoding($false)))
Write-Output ("dist/{0}.json  registry entry: {1}" -f $id, $what)
