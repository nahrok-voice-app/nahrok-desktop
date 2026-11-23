// MSIX paketi üretir. Kullanım:
//   node paketle.js          -> Store'a gidecek paket (imzasız; Store imzalar)
//   node paketle.js --yerel  -> bu bilgisayarda denemek için kendinden imzalı paket (yerel-kur.ps1 ile kurulur)
//
// Not: electron-builder önbelleği %LOCALAPPDATA% altında olunca Windows, makeappx.exe'nin yan yana (SxS)
// manifestlerini çözemiyor ("yan yana yapılandırması doğru değil"). Önbellek bu yüzden repo içine alınır.
const path = require('path');
const fs = require('fs');
const crypto = require('crypto');
const { execFileSync } = require('child_process');

const ONBELLEK = path.resolve(__dirname, '..', '..', '.eb-cache');
process.env.ELECTRON_BUILDER_CACHE = ONBELLEK;

const { build } = require('electron-builder');
const pkg = require('./package.json');

const YEREL_YAYIMCI = 'CN=Nahrok Yerel';

// Yalnız yerel deneme için: CurrentUser\My'de kendinden imzalı kod imzalama sertifikası (yoksa üretilir).
// Güven deposuna bir şey eklemez; o adımı kullanıcı yerel-kur.ps1 ile yönetici olarak yapar.
function yerelSertifika() {
  const pfx = path.join(ONBELLEK, 'yerel.pfx');
  const sifreDosyasi = path.join(ONBELLEK, 'yerel.pfx.sifre');
  const cer = path.join(__dirname, 'out', 'nahrok-yerel.cer');
  fs.mkdirSync(ONBELLEK, { recursive: true });
  fs.mkdirSync(path.dirname(cer), { recursive: true });
  if (!fs.existsSync(pfx)) {
    const sifre = crypto.randomBytes(18).toString('base64url');
    const ps = [
      `$c = New-SelfSignedCertificate -Type Custom -Subject '${YEREL_YAYIMCI}' -KeyUsage DigitalSignature`,
      `-FriendlyName 'Nahrok yerel deneme' -CertStoreLocation 'Cert:\\CurrentUser\\My'`,
      `-TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}') -NotAfter (Get-Date).AddYears(1);`,
      `Export-PfxCertificate -Cert $c -FilePath '${pfx}' -Password (ConvertTo-SecureString $env:PFX_SIFRE -AsPlainText -Force) | Out-Null`,
    ].join(' ');
    execFileSync('powershell.exe', ['-NoProfile', '-Command', ps], { env: { ...process.env, PFX_SIFRE: sifre }, stdio: 'inherit' });
    fs.writeFileSync(sifreDosyasi, sifre);
  }
  const sifre = fs.readFileSync(sifreDosyasi, 'utf8').trim();
  const ps = `$c = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2('${pfx}', $env:PFX_SIFRE); ` +
    `[IO.File]::WriteAllBytes('${cer}', $c.Export('Cert'))`;
  execFileSync('powershell.exe', ['-NoProfile', '-Command', ps], { env: { ...process.env, PFX_SIFRE: sifre }, stdio: 'inherit' });
  return { pfx, sifre };
}

const yerel = process.argv.includes('--yerel');
const config = { ...pkg.build, win: { ...pkg.build.win }, appx: { ...pkg.build.appx } };
if (yerel) {
  const { pfx, sifre } = yerelSertifika();
  config.appx.publisher = YEREL_YAYIMCI;
  config.appx.artifactName = 'nahrok-${version}-yerel.${ext}';
  config.win.signtoolOptions = { certificateFile: pfx, certificatePassword: sifre };
}

build({ win: ['appx'], publish: 'never', config })
  .then((dosyalar) => console.log('Paket:', dosyalar.filter((d) => /\.(appx|msix)$/i.test(d)).join(', ')))
  .catch((e) => { console.error(e); process.exit(1); });
