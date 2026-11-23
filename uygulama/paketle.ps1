# Nahrok MSIX paketi (PLAN.md çizgi 7 ve 9).
#   .\paketle.ps1          Store'a gidecek paket: imzasız (Store imzalar). Kimlik msix\magaza.json'dan.
#   .\paketle.ps1 -Yerel   Bu bilgisayarda denemek için kendinden imzalı paket; yerel-kur.ps1 ile (yönetici) kurulur.
#   -Exe <yol>             Derleme yerine hazır bir exe paketle (ör. derlemeyi başka makinede yaptıysan)
#   -DerlemeYok            target\release\nahrok.exe'yi yeniden derlemeden kullan
param([switch]$Yerel, [string]$Exe, [switch]$DerlemeYok)
$ErrorActionPreference = 'Stop'
$kok = $PSScriptRoot
$cikis = Join-Path $kok 'out'
New-Item -ItemType Directory -Force $cikis | Out-Null

# Windows SDK araçları (makeappx, makepri, signtool): en yeni sürüm
$sdk = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin' -Directory -Filter '10.*' |
  Where-Object { Test-Path (Join-Path $_.FullName 'x64\makeappx.exe') } | Sort-Object { [version]$_.Name } | Select-Object -Last 1
if (-not $sdk) { throw 'Windows SDK bulunamadı (makeappx.exe). Visual Studio Build Tools + Windows SDK kur.' }
$arac = Join-Path $sdk.FullName 'x64'

# 1) exe
if (-not $Exe) {
  $Exe = Join-Path $kok 'src-tauri\target\release\nahrok.exe'
  if (-not $DerlemeYok) {
    Push-Location $kok
    try { npx tauri build --no-bundle; if ($LASTEXITCODE) { throw "tauri build başarısız ($LASTEXITCODE)" } } finally { Pop-Location }
  }
}
if (-not (Test-Path $Exe)) { throw "exe yok: $Exe" }

# 2) kimlik
$conf = Get-Content (Join-Path $kok 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
$surum = ([version]$conf.version)
$msixSurum = "$($surum.Major).$($surum.Minor).$([Math]::Max(0,$surum.Build)).0"   # Store: dördüncü hane 0 olmalı
if ($Yerel) {
  $kimlik = 'Nahrok.Yerel'; $yayimci = 'CN=Nahrok Yerel'; $yayimciAdi = 'Nahrok (yerel deneme)'
} else {
  $m = Get-Content (Join-Path $kok 'msix\magaza.json') -Raw | ConvertFrom-Json
  if (-not $m.kimlik -or -not $m.yayimci -or -not $m.yayimciAdi) {
    throw 'msix\magaza.json boş: Partner Center > Ürün kimliği sayfasındaki Package/Identity/Name, Publisher ve PublisherDisplayName değerlerini yaz.'
  }
  $kimlik = $m.kimlik; $yayimci = $m.yayimci; $yayimciAdi = $m.yayimciAdi
}
$ek = if ($Yerel) { '-yerel' } else { '' }
$paket = Join-Path $cikis "nahrok-$($surum.ToString(3))$ek.msix"

# 3) içerik
$icerik = Join-Path $cikis "msix-icerik$ek"
if (Test-Path $icerik) { Remove-Item -Recurse -Force $icerik }
New-Item -ItemType Directory -Force (Join-Path $icerik 'Assets') | Out-Null
Copy-Item $Exe (Join-Path $icerik 'nahrok.exe')
foreach ($v in 'Square150x150Logo.png', 'Square44x44Logo.png', 'StoreLogo.png') {
  Copy-Item (Join-Path $kok "src-tauri\icons\$v") (Join-Path $icerik "Assets\$v")
}
$esc = { param($s) [Security.SecurityElement]::Escape($s) }
$manifest = (Get-Content (Join-Path $kok 'msix\AppxManifest.xml') -Raw) `
  -replace '\{KIMLIK\}', (& $esc $kimlik) -replace '\{YAYIMCI\}', (& $esc $yayimci) `
  -replace '\{YAYIMCI_ADI\}', (& $esc $yayimciAdi) -replace '\{SURUM\}', $msixSurum
[IO.File]::WriteAllText((Join-Path $icerik 'AppxManifest.xml'), $manifest, (New-Object Text.UTF8Encoding($false)))

# 4) kaynak dizini (resources.pri) ve paket
$pricfg = Join-Path $cikis "priconfig$ek.xml"
& "$arac\makepri.exe" createconfig /cf $pricfg /dq tr-TR /o | Out-Null
& "$arac\makepri.exe" new /pr $icerik /cf $pricfg /of (Join-Path $icerik 'resources.pri') /o | Out-Null
if ($LASTEXITCODE) { throw "makepri başarısız ($LASTEXITCODE)" }
& "$arac\makeappx.exe" pack /o /d $icerik /p $paket | Out-Null
if ($LASTEXITCODE) { throw "makeappx başarısız ($LASTEXITCODE)" }

# 5) yerel deneme: kendinden imzalı sertifikayla imzala (yalnız CurrentUser\My; güven deposuna dokunmaz)
if ($Yerel) {
  $sert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq $yayimci -and $_.NotAfter -gt (Get-Date).AddDays(7) } |
    Sort-Object NotAfter | Select-Object -Last 1
  if (-not $sert) {
    $sert = New-SelfSignedCertificate -Type Custom -Subject $yayimci -KeyUsage DigitalSignature -FriendlyName 'Nahrok yerel deneme' `
      -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddYears(1) `
      -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
  }
  [IO.File]::WriteAllBytes((Join-Path $cikis 'nahrok-yerel.cer'), $sert.Export('Cert'))
  & "$arac\signtool.exe" sign /fd SHA256 /sha1 $sert.Thumbprint /s My $paket | Out-Null
  if ($LASTEXITCODE) { throw "signtool başarısız ($LASTEXITCODE)" }
}

# 6) çizgi 9: antivirüs taraması (Defender). İşaretlenirse paket kullanılmaz.
$mp = Join-Path $env:ProgramFiles 'Windows Defender\MpCmdRun.exe'
if (Test-Path $mp) {
  & $mp -Scan -ScanType 3 -File $paket -DisableRemediation | Out-Null
  if ($LASTEXITCODE -eq 2) { Remove-Item $paket -Force; throw 'Microsoft Defender paketi işaretledi: gönderme, yanlış alarm bildir (PLAN.md çizgi 9).' }
  "Defender taraması temiz."
} else { Write-Warning 'MpCmdRun bulunamadı: Defender taraması yapılmadı.' }

"Paket: $paket ($msixSurum)"
if (-not $Yerel) { 'Store''a göndermeden önce çoklu antivirüs taramasından da geçir (PLAN.md çizgi 9).' }
