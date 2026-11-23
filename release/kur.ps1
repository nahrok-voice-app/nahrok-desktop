# Nahrok deneme sürümünü kurar ya da kaldırır. Kur.cmd ve Kaldır.cmd bunu çalıştırır (ayrıntı: README.md).
#   kur.ps1             deneme sertifikasını tanıtır (bir kez, yönetici izni ister) ve paketi kurar ya da günceller
#   kur.ps1 -Uninstall  paketi ve deneme sertifikasını kaldırır
param([switch]$Uninstall, [switch]$AddTrust, [switch]$RemoveTrust)
$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$certFile = Join-Path $here 'nahrok-test.cer'
$packageName = 'Nahrok.Yerel'
$thumbprint = (New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($certFile)).Thumbprint
$trustPath = "Cert:\LocalMachine\TrustedPeople\$thumbprint"

# Yönetici olarak çalışan kısım: yalnız sertifika güveni (paketin kendisi kullanıcının hesabına kurulur)
if ($AddTrust) { Import-Certificate -FilePath $certFile -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople' | Out-Null; return }
if ($RemoveTrust) { Remove-Item $trustPath -ErrorAction SilentlyContinue; return }

function Invoke-AsAdmin([string]$flag) {
  try {
    Start-Process powershell.exe -Verb RunAs -Wait -WindowStyle Hidden `
      -ArgumentList @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$PSCommandPath`"", $flag)
  } catch {
    throw 'Yönetici izni verilmedi.'
  }
}

if ($Uninstall) {
  Get-AppxPackage -Name $packageName | Remove-AppxPackage
  if (Test-Path $trustPath) {
    'Deneme sertifikası kaldırılıyor; açılan pencerede Evet''e bas.'
    Invoke-AsAdmin '-RemoveTrust'
  }
  'Nahrok ve deneme sertifikası kaldırıldı.'
  return
}

$package = Get-ChildItem $here -Filter 'nahrok-*-test.msix' | Sort-Object LastWriteTime | Select-Object -Last 1
if (-not $package) { throw 'Bu klasörde Nahrok paketi (nahrok-…-test.msix) yok.' }
Get-ChildItem $here | Unblock-File   # internetten indirilen dosyaların işaretini kaldır

if (-not (Test-Path $trustPath)) {
  'Nahrok deneme sertifikası Windows''a tanıtılıyor; açılan pencerede Evet''e bas. Bu yalnız ilk kurulumda sorulur.'
  Invoke-AsAdmin '-AddTrust'
  if (-not (Test-Path $trustPath)) { throw 'Sertifika eklenemedi.' }
}

"Kuruluyor: $($package.Name)"
Add-AppxPackage -Path $package.FullName -ForceUpdateFromAnyVersion -ForceApplicationShutdown
'Nahrok kuruldu. Başlat menüsünde "Nahrok" diye aratıp açabilirsin.'
