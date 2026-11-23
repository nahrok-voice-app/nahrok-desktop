# Nahrok MSIX paketi (PLAN.md çizgi 7 ve 9).
#   npm run package        Store'a gidecek paket: imzasız (Store imzalar), kimlik msix\store.json'dan. Çıktı: out\
#   npm run package:test   Arkadaşlarla deneme paketi: deneme sertifikasıyla imzalı. Çıktı: release\ (repoya girer)
# Seçenekler: -Exe <yol> derleme yerine hazır exe paketler; -NoBuild target\release\nahrok.exe'yi yeniden derlemez.
param([switch]$Test, [string]$Exe, [switch]$NoBuild)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$work = Join-Path $root 'out'
$release = Join-Path $root 'release'
New-Item -ItemType Directory -Force $work | Out-Null

# Windows SDK araçları (makeappx, makepri, signtool): en yeni sürüm
$sdk = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin' -Directory -Filter '10.*' |
  Where-Object { Test-Path (Join-Path $_.FullName 'x64\makeappx.exe') } | Sort-Object { [version]$_.Name } | Select-Object -Last 1
if (-not $sdk) { throw 'Windows SDK bulunamadı (makeappx.exe). Visual Studio Build Tools ile Windows SDK''yı kur.' }
$tools = Join-Path $sdk.FullName 'x64'

# 1) exe
if (-not $Exe) {
  $Exe = Join-Path $root 'src-tauri\target\release\nahrok.exe'
  if (-not $NoBuild) {
    Push-Location $root
    try { npx tauri build --no-bundle; if ($LASTEXITCODE) { throw "tauri build başarısız ($LASTEXITCODE)" } } finally { Pop-Location }
  }
}
if (-not (Test-Path $Exe)) { throw "exe bulunamadı: $Exe" }

# 2) kimlik. Deneme paketinin kimliği ve sertifikası sabit kalır: yeni sürüm eskisinin üstüne kurulur.
$config = Get-Content (Join-Path $root 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
$version = [version]$config.version
$msixVersion = "$($version.Major).$($version.Minor).$([Math]::Max(0, $version.Build)).0"   # Store: dördüncü hane 0 olmalı
if ($Test) {
  $identity = 'Nahrok.Yerel'; $publisher = 'CN=Nahrok Yerel'; $publisherName = 'Nahrok (deneme sürümü)'
  $outDir = $release; $suffix = '-test'
} else {
  $store = Get-Content (Join-Path $root 'msix\store.json') -Raw | ConvertFrom-Json
  if (-not $store.identity -or -not $store.publisher -or -not $store.publisherName) {
    throw 'msix\store.json boş: Partner Center > Ürün kimliği sayfasındaki Package/Identity/Name, Publisher ve PublisherDisplayName değerlerini yaz.'
  }
  $identity = $store.identity; $publisher = $store.publisher; $publisherName = $store.publisherName
  $outDir = $work; $suffix = ''
}
$package = Join-Path $outDir "nahrok-$($version.ToString(3))$suffix.msix"

# 3) paket içeriği
$content = Join-Path $work "msix-content$suffix"
if (Test-Path $content) { Remove-Item -Recurse -Force $content }
New-Item -ItemType Directory -Force (Join-Path $content 'Assets') | Out-Null
Copy-Item $Exe (Join-Path $content 'nahrok.exe')
foreach ($logo in 'Square150x150Logo.png', 'Square44x44Logo.png', 'StoreLogo.png') {
  Copy-Item (Join-Path $root "src-tauri\icons\$logo") (Join-Path $content "Assets\$logo")
}
$escape = { param($s) [Security.SecurityElement]::Escape($s) }
$manifest = (Get-Content (Join-Path $root 'msix\AppxManifest.xml') -Raw -Encoding UTF8) `
  -replace '\{IDENTITY\}', (& $escape $identity) -replace '\{PUBLISHER\}', (& $escape $publisher) `
  -replace '\{PUBLISHER_NAME\}', (& $escape $publisherName) -replace '\{VERSION\}', $msixVersion
[IO.File]::WriteAllText((Join-Path $content 'AppxManifest.xml'), $manifest, (New-Object Text.UTF8Encoding($false)))

# 4) kaynak dizini (resources.pri) ve paket
$priConfig = Join-Path $work "priconfig$suffix.xml"
& "$tools\makepri.exe" createconfig /cf $priConfig /dq tr-TR /o | Out-Null
& "$tools\makepri.exe" new /pr $content /cf $priConfig /of (Join-Path $content 'resources.pri') /o | Out-Null
if ($LASTEXITCODE) { throw "makepri başarısız ($LASTEXITCODE)" }
if ($Test) { Get-ChildItem $release -Filter 'nahrok-*-test.msix' | Remove-Item -Force }   # eski deneme paketleri
& "$tools\makeappx.exe" pack /o /d $content /p $package | Out-Null
if ($LASTEXITCODE) { throw "makeappx başarısız ($LASTEXITCODE)" }

# 5) deneme paketi: kendinden imzalı sertifikayla imzala (yalnız CurrentUser\My; güven deposuna dokunmaz)
if ($Test) {
  $cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq $publisher -and $_.NotAfter -gt (Get-Date).AddDays(30) } |
    Sort-Object NotAfter | Select-Object -Last 1
  if (-not $cert) {
    $cert = New-SelfSignedCertificate -Type Custom -Subject $publisher -KeyUsage DigitalSignature -FriendlyName 'Nahrok deneme paketi' `
      -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddYears(2) `
      -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
  }
  [IO.File]::WriteAllBytes((Join-Path $release 'nahrok-test.cer'), $cert.Export('Cert'))
  & "$tools\signtool.exe" sign /fd SHA256 /sha1 $cert.Thumbprint /s My $package | Out-Null
  if ($LASTEXITCODE) { throw "signtool başarısız ($LASTEXITCODE)" }
}

# 6) çizgi 9: antivirüs taraması (Defender). İşaretlenirse paket kullanılmaz.
$defender = Join-Path $env:ProgramFiles 'Windows Defender\MpCmdRun.exe'
if (Test-Path $defender) {
  & $defender -Scan -ScanType 3 -File $package -DisableRemediation | Out-Null
  if ($LASTEXITCODE -eq 2) {
    Remove-Item $package -Force
    throw 'Microsoft Defender paketi işaretledi: gönderme, yanlış alarm bildir (PLAN.md çizgi 9).'
  }
  'Defender taraması temiz.'
} else { Write-Warning 'MpCmdRun bulunamadı: Defender taraması yapılmadı.' }

"Paket: $package ($msixVersion)"
if (-not $Test) { 'Store''a göndermeden önce çoklu antivirüs taramasından da geçir (PLAN.md çizgi 9).' }
