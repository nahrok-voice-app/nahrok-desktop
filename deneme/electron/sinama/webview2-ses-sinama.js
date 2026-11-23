// YALNIZ SINAMA: aynı ölçümü Tauri deneme uygulamasında (WebView2) yapar.
// Çalıştırma: node sinama/webview2-ses-sinama.js <tauri-exe-yolu> [ek WebView2 bayrakları]
// Seçici WebView2'nin kendi penceresidir; ekran ve "sistem sesini paylaş" elle seçilir (ya da bayrakla otomatik).
const http = require('http');
const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawn } = require('child_process');

const [exe, ...ekBayrak] = process.argv.slice(2);
if (!exe) { console.error('Kullanım: node webview2-ses-sinama.js <exe> [bayraklar]'); process.exit(2); }
const SURE_SN = Number(process.env.SINAMA_SURE || 90);

function disSesCal(sn) {
  const hz = 44100, n = hz * sn, b = Buffer.alloc(44 + n * 2);
  b.write('RIFF', 0); b.writeUInt32LE(36 + n * 2, 4); b.write('WAVEfmt ', 8); b.writeUInt32LE(16, 16);
  b.writeUInt16LE(1, 20); b.writeUInt16LE(1, 22); b.writeUInt32LE(hz, 24); b.writeUInt32LE(hz * 2, 28);
  b.writeUInt16LE(2, 32); b.writeUInt16LE(16, 34); b.write('data', 36); b.writeUInt32LE(n * 2, 40);
  for (let i = 0; i < n; i++) b.writeInt16LE(Math.round(Math.sin(2 * Math.PI * 1000 * i / hz) * 1600), 44 + i * 2);
  const dosya = path.join(os.tmpdir(), 'nahrok-ses-sinama-1000hz-uzun.wav');
  fs.writeFileSync(dosya, b);
  return spawn('powershell.exe', ['-NoProfile', '-Command', `(New-Object Media.SoundPlayer '${dosya}').PlaySync()`], { stdio: 'ignore' });
}

const html = fs.readFileSync(path.join(__dirname, 'ses-sinama.html'));
let uygulama, oynatici;
const bitir = (kod) => { try { uygulama.kill(); } catch {} try { oynatici.kill(); } catch {} process.exit(kod); };

const sunucu = http.createServer((req, res) => {
  if (req.method === 'POST' && req.url === '/sonuc') {
    let s = ''; req.on('data', (d) => (s += d));
    req.on('end', () => { res.end('ok'); console.log(JSON.stringify(JSON.parse(s), null, 2)); bitir(0); });
    return;
  }
  res.setHeader('Content-Type', 'text/html; charset=utf-8'); res.end(html);
});

sunucu.listen(0, '127.0.0.1', () => {
  const url = `http://localhost:${sunucu.address().port}/?oto`;
  oynatici = disSesCal(SURE_SN);
  uygulama = spawn(exe, [], {
    stdio: 'ignore',
    env: { ...process.env, NAHROK_URL: url, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: ['--autoplay-policy=no-user-gesture-required', ...ekBayrak].join(' ') },
  });
  console.error(`Açıldı: ${url} (en çok ${SURE_SN} sn)`);
  setTimeout(() => { console.error('Zaman aşımı: sonuç gelmedi'); bitir(1); }, SURE_SN * 1000);
});
