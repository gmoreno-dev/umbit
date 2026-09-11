# umbit · instalador para windows 10 e 11
#
#   irm https://raw.githubusercontent.com/gmoreno-dev/umbit/main/install.ps1 | iex
#
# baixa o instalador da última release e roda a instalação silenciosa,
# só para o usuário atual (não pede administrador).
$ErrorActionPreference = 'Stop'
$repo = 'gmoreno-dev/umbit'

Write-Host 'umbit · procurando a última release' -ForegroundColor White
$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
$asset = $release.assets | Where-Object { $_.name -like '*-setup.exe' } | Select-Object -First 1
if (-not $asset) { throw "a release $($release.tag_name) não tem instalador para windows" }
Write-Host "versão $($release.tag_name)"

$tmp = Join-Path $env:TEMP $asset.name
Write-Host 'baixando o instalador'
Invoke-WebRequest $asset.browser_download_url -OutFile $tmp

Write-Host 'instalando (sem janelas; leva alguns segundos)'
$p = Start-Process -FilePath $tmp -ArgumentList '/S' -Wait -PassThru
if ($p.ExitCode -ne 0) { throw "o instalador saiu com código $($p.ExitCode)" }
Remove-Item $tmp -ErrorAction SilentlyContinue

Write-Host 'pronto. o umbit está no menu iniciar.' -ForegroundColor Green
Write-Host 'lembre: precisa de spotify premium e do seu próprio client id.'
Write-Host "passo a passo: https://github.com/$repo#seu-próprio-client-id-necessário"
