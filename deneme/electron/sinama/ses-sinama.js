// YALNIZ SINAMA: sistem sesiyle yayın ve restrictOwnAudio davranışını ölçer. Pakete girmez.
// Seçici burada otomatik geçilir (birinci ekran); gerçek uygulamada kullanıcı seçer (PLAN.md çizgi 4).
// Çalıştırma: npx electron sinama/ses-sinama.js [loopback|loopbackWithMute]
const { app, BrowserWindow, desktopCapturer, session } = require('electron');
const path = require('path');
const fs = require('fs');
const os = require('os');
const { spawn } = require('child_process');

const SES_MODU = process.argv.find((a) => a.startsWith('loopback')) || 'loopback';
// Denemek için Chromium özellik bayrakları: SINAMA_OZELLIK=RestrictOwnAudio,RestrictOwnAudioAddChromiumBack
if (process.env.SINAMA_OZELLIK) app.commandLine.appendSwitch('enable-features', process.env.SINAMA_OZELLIK);

// Başka bir uygulamanın sesi: 1000 Hz, 12 sn'lik WAV, ayrı PowerShell işleminde çalınır.
function disSesCal() {
  const hz = 44100, sn = 12, n = hz * sn, b = Buffer.alloc(44 + n * 2);
  b.write('RIFF', 0); b.writeUInt32LE(36 + n * 2, 4); b.write('WAVEfmt ', 8); b.writeUInt32LE(16, 16);
  b.writeUInt16LE(1, 20); b.writeUInt16LE(1, 22); b.writeUInt32LE(hz, 24); b.writeUInt32LE(hz * 2, 28);
  b.writeUInt16LE(2, 32); b.writeUInt16LE(16, 34); b.write('data', 36); b.writeUInt32LE(n * 2, 40);
  for (let i = 0; i < n; i++) b.writeInt16LE(Math.round(Math.sin(2 * Math.PI * 1000 * i / hz) * 1600), 44 + i * 2);
  const dosya = path.join(os.tmpdir(), 'nahrok-ses-sinama-1000hz.wav');
  fs.writeFileSync(dosya, b);
  return spawn('powershell.exe', ['-NoProfile', '-Command', `(New-Object Media.SoundPlayer '${dosya}').PlaySync()`], { stdio: 'ignore' });
}

app.whenReady().then(async () => {
  session.defaultSession.setPermissionRequestHandler((_wc, _izin, cb) => cb(true));
  session.defaultSession.setDisplayMediaRequestHandler(async (_istek, cb) => {
    const [ekran] = await desktopCapturer.getSources({ types: ['screen'] });
    cb({ video: ekran, audio: SES_MODU });
  });

  const w = new BrowserWindow({ show: false, webPreferences: { sandbox: true, contextIsolation: true } });
  await w.loadFile(path.join(__dirname, 'ses-sinama.html'));
  const oynatici = disSesCal();
  await new Promise((r) => setTimeout(r, 1500));
  const sonuc = await w.webContents.executeJavaScript('sina()');
  oynatici.kill();
  console.log(JSON.stringify({ electron: process.versions.electron, chrome: process.versions.chrome, sesModu: SES_MODU, sonuc }, null, 2));
  app.quit();
});
