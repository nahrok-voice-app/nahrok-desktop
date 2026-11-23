# Yerel deneme paketini bu bilgisayara kurar. YÖNETİCİ PowerShell'de:
#   powershell -ExecutionPolicy Bypass -File .\yerel-kur.ps1          (önce: npm run paketle:yerel)
#   powershell -ExecutionPolicy Bypass -File .\yerel-kur.ps1 -Kaldir   (paketi ve test sertifikasını kaldırır)
# Ne yapar: out\nahrok-yerel.cer test sertifikasını LocalMachine\TrustedPeople'a ekler (Windows MSIX kurulumu için ister),
# sonra paketi kurar. Store sürümünde buna gerek yoktur; paketi Microsoft imzalar.
param([switch]$Kaldir)
$ErrorActionPreference = 'Stop'
$cer = Join-Path $PSScriptRoot 'out\nahrok-yerel.cer'
if (-not (Test-Path $cer)) { throw 'Sertifika yok. Önce: npm run paketle:yerel' }
$parmakIzi = (New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($cer)).Thumbprint

if ($Kaldir) {
  Get-AppxPackage -Name 'Nahrok.Yerel' | Remove-AppxPackage
  Remove-Item "Cert:\LocalMachine\TrustedPeople\$parmakIzi" -ErrorAction SilentlyContinue
  'Paket ve test sertifikası kaldırıldı.'
  return
}

$paket = Get-ChildItem (Join-Path $PSScriptRoot 'out') -Filter 'nahrok-*-yerel.msix' | Sort-Object LastWriteTime | Select-Object -Last 1
if (-not $paket) { throw 'Paket yok. Önce: npm run paketle:yerel' }
Import-Certificate -FilePath $cer -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople' | Out-Null
Add-AppxPackage -Path $paket.FullName -ForceUpdateFromAnyVersion
"Kuruldu: $($paket.Name). Başlat menüsünde 'Nahrok'."
