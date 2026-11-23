# Yerel deneme paketini bu bilgisayara kurar. YÖNETİCİ PowerShell'de çalıştır:
#   powershell -ExecutionPolicy Bypass -File .\yerel-kur.ps1
# Önce: npm run paketle:yerel
#
# Ne yapar: out\nahrok-yerel.cer test sertifikasını LocalMachine\TrustedPeople deposuna ekler (MSIX kurulumu bunu ister),
# sonra paketi kurar. Geri almak için: .\yerel-kur.ps1 -Kaldir
param([switch]$Kaldir)
$ErrorActionPreference = 'Stop'
$cer = Join-Path $PSScriptRoot 'out\nahrok-yerel.cer'
$parmakIzi = (New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($cer)).Thumbprint

if ($Kaldir) {
  Get-AppxPackage -Name 'Nahrok.Deneme' | Remove-AppxPackage
  Remove-Item "Cert:\LocalMachine\TrustedPeople\$parmakIzi" -ErrorAction SilentlyContinue
  'Paket ve test sertifikası kaldırıldı.'
  return
}

$paket = Get-ChildItem (Join-Path $PSScriptRoot 'out') -Filter 'nahrok-*-yerel.appx' | Sort-Object LastWriteTime | Select-Object -Last 1
if (-not $paket) { throw 'Paket yok. Önce: npm run paketle:yerel' }
Import-Certificate -FilePath $cer -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople' | Out-Null
Add-AppxPackage -Path $paket.FullName -ForceUpdateFromAnyVersion
"Kuruldu: $($paket.Name). Başlat menüsünde 'Nahrok Deneme'."
