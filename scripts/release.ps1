# RustPlanetX - расширение RustGIS для PlanetX Lite.
# Copyright (C) 2026 ООО «Информ++». Лицензия Apache 2.0.
# SPDX-License-Identifier: Apache-2.0
#
# Выпуск RustPlanetX одной командой, решение автора от 10 ноября 2026 года:
#
#   powershell -File scripts/release.ps1 -Version 0.5.7 -Notes "что нового"
#
# Шаги:
#   1. Коммит PlanetX Lite - последний в origin/main (или -LiteRef), его CI
#      должен быть зелёным.
#   2. lite.ref и номер в plugin.json, коммит, метка v<номер>, отправка.
#   3. Ожидание сборки Release на GitHub, сверка пакета: SHA-256 записи
#      реестра и номер в plugin.json пакета.
#   4. Dropbox: RustPlanetX.rgplugin и ВЕРСИЯ.txt.
#   5. Каталог RustGIS: запись реестра в форке Valery35/rustgis-plugins -
#      в ветку открытой заявки или в новую ветку (тогда ссылка на заявку).
#   6. -Install: выпуск ставится в RustGIS (rustgis-cli plugins install).
#
# -DryRun только проверяет и печатает, что будет сделано. Публикация на
# площадках - отдельно, навык publish-release.

param(
    [Parameter(Mandatory = $true)][string]$Version,
    [string]$Notes = "",
    [string]$LiteRef = "",
    [string]$Lite = "C:\Dev\planetx-lite",
    [string]$Dropbox = (Join-Path $env:USERPROFILE "Dropbox\ИИ\RustPlanetX"),
    [string]$ForkDir = (Join-Path $env:LOCALAPPDATA "rustplanetx-release\rustgis-plugins"),
    [switch]$Install,
    [switch]$SkipCatalog,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$root = Split-Path -Parent $PSScriptRoot
$api = 'https://api.github.com/repos'
$headers = @{ 'User-Agent' = 'rustplanetx-release' }

function Say([string]$text) { Write-Output "== $text" }

# git - из PATH или из GitHub Desktop: у автора git в PATH нет.
function Find-Git {
    $cmd = Get-Command git -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $desk = Get-ChildItem (Join-Path $env:LOCALAPPDATA 'GitHubDesktop') -Directory -Filter 'app-*' -ErrorAction SilentlyContinue | Sort-Object Name -Descending
    foreach ($d in $desk) {
        $g = Join-Path $d.FullName 'resources\app\git\cmd\git.exe'
        if (Test-Path $g) { return $g }
    }
    throw 'git не найден: ни в PATH, ни в GitHub Desktop'
}
$git = Find-Git

function Git([string]$dir) {
    $out = & $git -C $dir @args 2>&1
    if ($LASTEXITCODE -ne 0) { throw "git $($args -join ' '): $out" }
    return $out
}

if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "номер выпуска - три числа через точку, а не '$Version'" }
$tag = "v$Version"

# 1. Хранилище rustplanetx - чистое и совпадает с origin/main.
Say "rustplanetx: проверка"
Git $root fetch -q origin | Out-Null
$dirty = Git $root status --porcelain
if ($dirty) {
    $text = "в rustplanetx есть несохранённые правки:`n$($dirty -join "`n")"
    if ($DryRun) { Write-Warning $text } else { throw $text }
}
$behind = Git $root rev-list --count 'HEAD..origin/main'
if ([int]$behind -gt 0) { throw "rustplanetx отстаёт от origin/main на $behind коммитов" }
if ((Git $root tag -l $tag)) { throw "метка $tag уже есть" }
$manifestPath = Join-Path $root 'plugin.json'
$manifestText = [IO.File]::ReadAllText($manifestPath)
$current = ([regex]::Match($manifestText, '"version"\s*:\s*"([^"]+)"')).Groups[1].Value
if ([version]$Version -le [version]$current) { throw "номер $Version не больше нынешнего $current" }

# Коммит Lite и его CI.
Say "PlanetX Lite: коммит"
Git $Lite fetch -q origin | Out-Null
if ($LiteRef) {
    $sha = (Git $Lite rev-parse "$LiteRef^{commit}").Trim()
} else {
    # CI Lite не запускается на коммиты только с текстами, поэтому берётся
    # последний коммит main с зелёным CI, а не вершина main.
    $green = (Invoke-RestMethod "$api/Valery35/planetx-lite/actions/runs?branch=main&status=success&per_page=50" -Headers $headers).workflow_runs | Where-Object { $_.name -eq 'CI' } | Select-Object -First 1
    if (-not $green) { throw 'в main PlanetX Lite нет коммита с зелёным CI' }
    $sha = $green.head_sha
    $tip = (Git $Lite rev-parse 'origin/main').Trim()
    if ($tip -ne $sha) {
        $after = Git $Lite log --oneline "$sha..$tip"
        Say "после него в main Lite без CI: $(@($after).Count) коммитов, последний $($tip.Substring(0, 7))"
    }
}
$short = $sha.Substring(0, 7)
$liteVersion = ([regex]::Match((Git $Lite show "${sha}:Cargo.toml") -join "`n", '(?m)^version\s*=\s*"([^"]+)"')).Groups[1].Value
$runs = (Invoke-RestMethod "$api/Valery35/planetx-lite/actions/runs?head_sha=$sha" -Headers $headers).workflow_runs | Where-Object { $_.name -eq 'CI' }
$ci = $runs | Select-Object -First 1
if (-not $ci) { Write-Warning "у коммита Lite $short ещё нет прогона CI" }
elseif ($ci.status -ne 'completed') { Write-Warning "CI коммита Lite $short ещё идёт: $($ci.status)" }
elseif ($ci.conclusion -ne 'success') { throw "CI коммита Lite $short не зелёный: $($ci.conclusion)" }
Say "Lite $liteVersion, коммит $short, CI: $(if ($ci) { "$($ci.status) $($ci.conclusion)" } else { 'нет' })"

$oldRef = ([IO.File]::ReadAllText((Join-Path $root 'lite.ref'))).Trim()
if ($oldRef -eq $sha) { Write-Warning "lite.ref не меняется: программа та же, что в $current" }
$title = if ($Notes) { "RustPlanetX ${Version}: $Notes (Lite $short)" } else { "RustPlanetX ${Version} (Lite $short)" }
Say "Выпуск $tag : $current -> $Version, lite.ref $($oldRef.Substring(0,7)) -> $short"
if ($DryRun) {
    Say "DryRun: коммит «$title», метка $tag, сборка, Dropbox $Dropbox, каталог$(if ($SkipCatalog) { ' - пропуск' } else { " ($ForkDir)" })$(if ($Install) { ', установка в RustGIS' })"
    $prs = (Invoke-RestMethod "$api/opengeos/rustgis-plugins/pulls?state=open&per_page=50" -Headers $headers) | Where-Object { $_.user.login -eq 'Valery35' -and $_.head.ref -like 'rustplanetx*' }
    Say "Открытая заявка в каталог: $(if ($prs) { ($prs | ForEach-Object { "#$($_.number) ветка $($_.head.ref)" }) -join ', ' } else { 'нет, будет новая ветка' })"
    return
}

# 2. lite.ref, plugin.json, коммит, метка.
[IO.File]::WriteAllText((Join-Path $root 'lite.ref'), "$sha`n", (New-Object Text.UTF8Encoding($false)))
$manifestText = [regex]::Replace($manifestText, '("version"\s*:\s*")[^"]+(")', "`${1}$Version`${2}", 1)
[IO.File]::WriteAllText($manifestPath, $manifestText, (New-Object Text.UTF8Encoding($false)))
$msgFile = Join-Path ([IO.Path]::GetTempPath()) "rustplanetx-release-$Version.txt"
[IO.File]::WriteAllText($msgFile, "$title`n", (New-Object Text.UTF8Encoding($false)))
Git $root add lite.ref plugin.json | Out-Null
Git $root commit -q -F $msgFile | Out-Null
Git $root tag $tag | Out-Null
Git $root push -q origin main $tag | Out-Null
Remove-Item $msgFile -ErrorAction SilentlyContinue
Say "Метка $tag отправлена"

# 3. Сборка Release и сверка пакета.
Say "Ожидание сборки Release (до 60 минут)"
$deadline = (Get-Date).AddMinutes(60)
$run = $null
do {
    Start-Sleep 30
    $run = (Invoke-RestMethod "$api/Valery35/rustplanetx/actions/runs?per_page=10" -Headers $headers).workflow_runs | Where-Object { $_.head_branch -eq $tag -and $_.name -eq 'Release' } | Select-Object -First 1
} while ((-not $run -or $run.status -ne 'completed') -and (Get-Date) -lt $deadline)
if (-not $run -or $run.status -ne 'completed') { throw "сборка $tag не закончилась за час" }
if ($run.conclusion -ne 'success') { throw "сборка $tag : $($run.conclusion), $($run.html_url)" }
$rel = Invoke-RestMethod "$api/Valery35/rustplanetx/releases/tags/$tag" -Headers $headers
$work = Join-Path ([IO.Path]::GetTempPath()) "rustplanetx-$Version"
New-Item -ItemType Directory -Force $work | Out-Null
$pkgName = "rustplanetx-$Version.rgplugin"
foreach ($a in $rel.assets | Where-Object { $_.name -eq $pkgName -or $_.name -eq 'rustplanetx.json' }) {
    Invoke-WebRequest $a.browser_download_url -OutFile (Join-Path $work $a.name) -Headers $headers
}
$pkg = Join-Path $work $pkgName
$entryPath = Join-Path $work 'rustplanetx.json'
$entry = Get-Content $entryPath -Raw | ConvertFrom-Json
$pkgSha = (Get-FileHash $pkg -Algorithm SHA256).Hash.ToLower()
if ($entry.source.sha256 -ne $pkgSha) { throw "SHA-256 пакета $pkgSha не совпадает с записью реестра $($entry.source.sha256)" }
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::OpenRead($pkg)
try {
    $reader = New-Object IO.StreamReader(($zip.Entries | Where-Object FullName -eq 'plugin.json').Open())
    $inside = ($reader.ReadToEnd() | ConvertFrom-Json).version
    $reader.Close()
} finally { $zip.Dispose() }
if ($inside -ne $Version) { throw "в пакете plugin.json версии $inside, а не $Version" }
Say "Пакет сверен: $pkgSha"

# 4. Dropbox.
if (Test-Path $Dropbox) {
    Copy-Item $pkg (Join-Path $Dropbox 'RustPlanetX.rgplugin') -Force
    $versionFile = Join-Path $Dropbox 'ВЕРСИЯ.txt'
    $months = 'января', 'февраля', 'марта', 'апреля', 'мая', 'июня', 'июля', 'августа', 'сентября', 'октября', 'ноября', 'декабря'
    $now = Get-Date
    $day = "$($now.Day) $($months[$now.Month - 1]) $($now.Year) года"
    $head = @(
        "RustPlanetX $Version - расширение RustGIS, глобус PlanetX со связью с картой RustGIS",
        "$day. Внутри программа RustPlanetX из PlanetX Lite $liteVersion, коммит $short, собрана GitHub Actions",
        "(выпуск github.com/Valery35/rustplanetx/releases/tag/$tag)."
    )
    if ($Notes) { $head += "${Version}: $Notes." }
    $old = if (Test-Path $versionFile) { [IO.File]::ReadAllLines($versionFile) } else { @() }
    # Прежняя шапка - три первые строки, история ниже остаётся.
    $rest = if ($old.Count -gt 3) { $old[3..($old.Count - 1)] } else { @() }
    $rest = $rest | ForEach-Object { if ($_ -like 'SHA-256 пакета:*') { "SHA-256 пакета: $pkgSha" } else { $_ } }
    [IO.File]::WriteAllLines($versionFile, [string[]]($head + $rest), (New-Object Text.UTF8Encoding($false)))
    Say "Dropbox обновлён: $Dropbox"
} else {
    Write-Warning "нет папки Dropbox $Dropbox"
}

# 5. Каталог RustGIS.
if (-not $SkipCatalog) {
    if (-not (Test-Path (Join-Path $ForkDir '.git'))) {
        New-Item -ItemType Directory -Force (Split-Path $ForkDir) | Out-Null
        & $git clone -q https://github.com/Valery35/rustgis-plugins.git $ForkDir
        if ($LASTEXITCODE -ne 0) { throw 'форк rustgis-plugins не клонируется' }
        Git $ForkDir remote add upstream https://github.com/opengeos/rustgis-plugins.git | Out-Null
    }
    Git $ForkDir fetch -q origin | Out-Null
    Git $ForkDir fetch -q upstream main | Out-Null
    $pr = (Invoke-RestMethod "$api/opengeos/rustgis-plugins/pulls?state=open&per_page=50" -Headers $headers) | Where-Object { $_.user.login -eq 'Valery35' -and $_.head.ref -like 'rustplanetx*' } | Select-Object -First 1
    $branch = if ($pr) { $pr.head.ref } else { "rustplanetx-$Version" }
    if ($pr) { Git $ForkDir checkout -q -B $branch "origin/$branch" | Out-Null } else { Git $ForkDir checkout -q -B $branch upstream/main | Out-Null }
    Copy-Item $entryPath (Join-Path $ForkDir 'registry\rustplanetx.json') -Force
    Git $ForkDir add registry/rustplanetx.json | Out-Null
    $changed = Git $ForkDir status --porcelain
    if ($changed) {
        $msg = if ($Notes) { "RustPlanetX ${Version}: $Notes" } else { "RustPlanetX $Version" }
        [IO.File]::WriteAllText($msgFile, "$msg`n", (New-Object Text.UTF8Encoding($false)))
        Git $ForkDir commit -q -F $msgFile | Out-Null
        Remove-Item $msgFile -ErrorAction SilentlyContinue
        Git $ForkDir push -q origin $branch | Out-Null
    }
    if ($pr) {
        Say "Заявка #$($pr.number) переведена на $Version, заголовок меняет автор: $($pr.html_url)"
    } else {
        Say "Новая заявка: https://github.com/opengeos/rustgis-plugins/compare/main...Valery35:rustgis-plugins:$branch"
    }
}

# 6. Установка в RustGIS.
if ($Install) {
    Get-Process planetx-lite -ErrorAction SilentlyContinue | ForEach-Object { $_.CloseMainWindow() | Out-Null }
    Start-Sleep 3
    $cli = Join-Path $env:LOCALAPPDATA 'Programs\RustGIS\rustgis-cli.exe'
    & $cli plugins install $pkg | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'rustgis-cli plugins install не прошёл' }
    Say "Выпуск $Version стоит в RustGIS"
}

Say "Готово: $($rel.html_url)"
