// Duman testi: hata ayıklama derlemesini yerel bir sahte siteye karşı çalıştırır; PLAN.md çizgi 2–4'ü ve v1
// davranışlarını sınar. Önce: npm run build:test   Sonra: npm run test:smoke
// Ekranda birkaç kez Nahrok penceresi açılıp kapanır; Ctrl+Shift+F11 tuşları bir kez gönderilir.
const http = require('http');
const fs = require('fs');
const path = require('path');
const { spawn, execFileSync } = require('child_process');

const EXE = process.argv[2] || path.join(__dirname, '..', 'src-tauri', 'target', 'debug', 'nahrok.exe');
// Sınama derlemesinin kimliği ayrıdır (tauri.test.conf.json): kurulu Nahrok'la çakışmaz, kullanıcının ayarlarına dokunmaz
const SETTINGS_FILE = path.join(process.env.APPDATA || '', 'com.nahrok.desktop.test', 'settings.json');
const VERSION = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'src-tauri', 'tauri.conf.json'), 'utf8')).version;

// Kullanıcının kendi ayarları sınamadan sonra geri konur
const savedSettings = fs.existsSync(SETTINGS_FILE) ? fs.readFileSync(SETTINGS_FILE) : null;
const restoreSettings = () => {
  try { savedSettings ? fs.writeFileSync(SETTINGS_FILE, savedSettings) : fs.rmSync(SETTINGS_FILE, { force: true }); } catch { }
};
const writeSettings = (settings) => {
  fs.mkdirSync(path.dirname(SETTINGS_FILE), { recursive: true });
  fs.writeFileSync(SETTINGS_FILE, JSON.stringify(settings));
};

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const check = (name, passed, detail = '') => {
  results.push({ name, passed: !!passed });
  console.log(`${passed ? '✔' : '✖'} ${name}${detail ? `  (${detail})` : ''}`);
};

const ps = (script) => execFileSync('powershell.exe', ['-NoProfile', '-Command', script], { encoding: 'utf8' }).trim();
const isRunning = (pid) => { try { process.kill(pid, 0); return true; } catch { return false; } };
const requestClose = (pid) => ps(`$p=Get-Process -Id ${pid} -ErrorAction SilentlyContinue; if ($p) { $p.CloseMainWindow() | Out-Null }`);

// Ana pencere "Tauri Window" sınıfındaki "Nahrok" başlıklı penceredir. MainWindowHandle kullanılmaz: ana pencere
// gizlenince .NET onun yerine tek kopya eklentisinin görünmez ileti penceresini döndürür.
const WIN32 = `Add-Type -Namespace W -Name U -MemberDefinition @'
public delegate bool CB(System.IntPtr h, System.IntPtr l);
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool EnumWindows(CB cb, System.IntPtr l);
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(System.IntPtr h, out uint p);
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool IsWindowVisible(System.IntPtr h);
[System.Runtime.InteropServices.DllImport("user32.dll", CharSet=System.Runtime.InteropServices.CharSet.Unicode)] public static extern int GetClassName(System.IntPtr h, System.Text.StringBuilder s, int n);
[System.Runtime.InteropServices.DllImport("user32.dll", CharSet=System.Runtime.InteropServices.CharSet.Unicode)] public static extern int GetWindowText(System.IntPtr h, System.Text.StringBuilder s, int n);
'@`;
const windowVisible = (pid, title = 'Nahrok') => ps(`${WIN32}
$script:r = 'yok'
[W.U]::EnumWindows({ param($h,$l) $p=0; [void][W.U]::GetWindowThreadProcessId($h,[ref]$p)
  $c=New-Object Text.StringBuilder 64; [void][W.U]::GetClassName($h,$c,64); $t=New-Object Text.StringBuilder 128; [void][W.U]::GetWindowText($h,$t,128)
  if ($p -eq ${pid} -and $c.ToString() -eq 'Tauri Window' -and $t.ToString() -eq '${title}') { $script:r = [W.U]::IsWindowVisible($h); return $false }; $true }, [IntPtr]::Zero) | Out-Null
$script:r`);

// UI Automation: süreç penceresindeki belge adı (sayfa başlığı) ya da bir penceredeki bütün metinler
const UIA = `Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes; $A=[Windows.Automation.AutomationElement]; $S=[Windows.Automation.TreeScope]`;
const documentName = (pid) => ps(`${UIA}
$w=$A::RootElement.FindFirst($S::Children,(New-Object Windows.Automation.PropertyCondition($A::ProcessIdProperty, ${pid})))
if ($w) { $d=$w.FindFirst($S::Descendants,(New-Object Windows.Automation.PropertyCondition($A::ControlTypeProperty,[Windows.Automation.ControlType]::Document))); if ($d) { $d.Current.Name } }`);
const windowTexts = (pid, title) => ps(`${UIA}
$c=New-Object Windows.Automation.AndCondition((New-Object Windows.Automation.PropertyCondition($A::ProcessIdProperty, ${pid})),(New-Object Windows.Automation.PropertyCondition($A::NameProperty, '${title}')))
$w=$A::RootElement.FindFirst($S::Children, $c)
if ($w) { ($w.FindAll($S::Descendants,[Windows.Automation.Condition]::TrueCondition) | ForEach-Object { $_.Current.Name } | Where-Object { $_ }) -join [char]10 }`);

// Sahte site: her yüklenişte sınamaları koşar, sonuçları /event'e yollar
const PAGE = `<!doctype html><meta charset="utf-8"><title>Duman</title><body>duman<script>
const send = (type, data) => fetch('/event', { method: 'POST', body: JSON.stringify({ type, data }) });
const d = window.nahrokDesktop;
const params = new URLSearchParams(location.search);
(async () => {
  const r = { bridge: !!d, frozen: !!d && Object.isFrozen(d), version: d && d.version };
  try { d.setBadge(3); d.setVoiceState({ inVoice: params.has('inVoice') }); r.calls = true; } catch (e) { r.calls = String(e); }
  const tryInvoke = async (command, args) => {
    try { await window.__TAURI_INTERNALS__.invoke(command, args); return 'izin verildi'; } catch { return 'reddedildi'; }
  };
  r.denied = {
    opener: await tryInvoke('plugin:opener|open_url', { url: 'https://example.com' }),
    globalShortcut: await tryInvoke('plugin:global-shortcut|unregister_all', {}),
    windowClose: await tryInvoke('plugin:window|close', {}),
    windowState: await tryInvoke('plugin:window-state|save_window_state', { flags: 0 }),
    getSettings: await tryInvoke('get_settings', {}),
    updateSettings: await tryInvoke('update_settings', { patch: { closeToTray: true } }),
    startAtLogin: await tryInvoke('set_start_at_login', { enabled: true }),
  };
  r.camera = await navigator.mediaDevices.getUserMedia({ video: true }).then(s => { s.getTracks().forEach(t => t.stop()); return 'verildi'; }, e => e.name);
  r.microphone = await navigator.mediaDevices.getUserMedia({ audio: true }).then(s => { s.getTracks().forEach(t => t.stop()); return 'verildi'; }, e => e.name);
  r.popup = window.open('about:blank') === null ? 'engellendi' : 'açıldı';
  d.onShortcut(action => send('shortcut', action));
  setInterval(() => send('heartbeat', document.visibilityState), 500);
  send('result', r);
  if (params.has('openSettings')) d.openSettings();
  if (params.has('navigateAway')) setTimeout(() => { location.href = 'file:///C:/Windows/win.ini'; }, 300);
})();
</script>`;

let events = [];
let minVersion = '0.0.1';
const server = http.createServer((req, res) => {
  if (req.method === 'POST' && req.url === '/event') {
    let body = '';
    req.on('data', (chunk) => (body += chunk));
    req.on('end', () => { events.push({ ...JSON.parse(body), time: Date.now() }); res.end(); });
    return;
  }
  if (req.url === '/desktop.json') {
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify({ minVersion }));
    return;
  }
  res.setHeader('Content-Type', 'text/html; charset=utf-8');
  res.end(PAGE);
});

const launch = (url) => spawn(EXE, [], { stdio: 'ignore', env: { ...process.env, NAHROK_URL: url } });
const waitForEvent = async (type, ms = 20000, after = 0) => {
  for (let t = 0; t < ms; t += 200) {
    const e = events.find((x) => x.type === type && x.time > after);
    if (e) return e;
    await sleep(200);
  }
  return null;
};
const stop = async (app) => { try { process.kill(app.pid); } catch { } await sleep(1000); };

(async () => {
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const site = `http://localhost:${server.address().port}/`;
  writeSettings({ toggleMute: 'Ctrl+Shift+F11' });

  // 1) köprü, izinler, gezinme
  const app = launch(`${site}?navigateAway`);
  const first = await waitForEvent('result');
  if (!first) { check('site yüklendi ve köprü yanıt verdi', false); await stop(app); restoreSettings(); process.exit(1); }
  const r = first.data;
  check('köprü var, değiştirilemez, sürüm doğru', r.bridge && r.frozen && r.version === VERSION, r.version);
  check('köprü çağrıları hata vermez', r.calls === true, String(r.calls));
  for (const [name, outcome] of Object.entries(r.denied)) check(`liste dışı komut reddedilir: ${name}`, outcome === 'reddedildi', outcome);
  check('kamera izni verilmez', r.camera !== 'verildi', r.camera);
  check('mikrofon izni verilir', r.microphone === 'verildi' || r.microphone === 'NotFoundError', r.microphone);
  check('yeni pencere açılmaz', r.popup === 'engellendi');
  await sleep(1500);
  const loads = events.filter((e) => e.type === 'result').length;
  check('başka adrese (file:) gezinme engellenir', loads === 1 && (await waitForEvent('heartbeat', 3000, Date.now() - 400)));

  // 2) ayar dosyasındaki kısayol (RegisterHotKey) siteye ulaşır
  ps(`(New-Object -ComObject WScript.Shell).SendKeys('^+{F11}')`);
  const shortcut = await waitForEvent('shortcut', 5000);
  check('ayarlardaki genel kısayol siteye ulaşır', shortcut && shortcut.data === 'toggleMute', shortcut && shortcut.data);

  // 3) tek kopya
  const second = launch(site);
  await sleep(4000);
  check('ikinci açılış kapanır, ilki sürer', second.exitCode !== null && isRunning(app.pid), `ikinci çıkış=${second.exitCode}`);

  // 4) seste değilken ve "kapatınca tepside" kapalıyken pencereyi kapatmak uygulamayı kapatır
  requestClose(app.pid);
  await sleep(2500);
  check('seste değilken pencereyi kapatmak uygulamayı kapatır', !isRunning(app.pid));

  // 5) seste iken kapatınca tepsiye iner (süreç ve sayfa sürer)
  events = [];
  const inVoice = launch(`${site}?inVoice`);
  await waitForEvent('result');
  await sleep(800);
  requestClose(inVoice.pid);
  await sleep(2000);
  const visible = windowVisible(inVoice.pid);
  const closedAt = Date.now();
  await sleep(2500);
  const heartbeat = events.some((e) => e.type === 'heartbeat' && e.time > closedAt);
  check('seste iken kapatınca tepsiye iner (süreç sürer, pencere gizli, sayfa çalışır)',
    isRunning(inVoice.pid) && visible === 'False' && heartbeat, `görünür=${visible} nabız=${heartbeat}`);
  await stop(inVoice);

  // 6) "kapatınca tepside çalışsın" ayarı: seste olmasa da tepsiye iner
  writeSettings({ toggleMute: 'Ctrl+Shift+F11', closeToTray: true });
  events = [];
  const toTray = launch(site);
  await waitForEvent('result');
  await sleep(800);
  requestClose(toTray.pid);
  await sleep(2000);
  check('"kapatınca tepside çalışsın" açıkken kapatınca tepsiye iner',
    isRunning(toTray.pid) && windowVisible(toTray.pid) === 'False', `görünür=${windowVisible(toTray.pid)}`);
  await stop(toTray);

  // 7) site ayarlar penceresini açabilir; pencere kendi komutlarıyla ayarları okur
  events = [];
  const withSettings = launch(`${site}?openSettings`);
  await waitForEvent('result');
  await sleep(4000);
  const settingsVisible = windowVisible(withSettings.pid, 'Nahrok ayarları');
  const texts = windowTexts(withSettings.pid, 'Nahrok ayarları');
  check('site ayarlar penceresini açar', settingsVisible === 'True', `görünür=${settingsVisible}`);
  check('ayarlar penceresi ayarları okur (sürüm, kısayol)',
    texts.includes(`Nahrok ${VERSION}`) && texts.includes('Ctrl+Shift+F11'), texts.split('\n').slice(-3).join(' | '));
  check('paketsiz sürümde "bilgisayar açılınca başlat" kapalı ve açıklamalı', texts.includes('yalnız kurulu sürümde'));
  await stop(withSettings);

  // 8) en düşük sürüm: site 99.0.0 isterse "güncelleme gerekli" sayfası
  minVersion = '99.0.0';
  const old = launch(site);
  await sleep(7000);
  const updateTitle = documentName(old.pid);
  check('eski sürüm "güncelleme gerekli" sayfasını görür', /güncelleme gerekli/i.test(updateTitle), updateTitle);
  await stop(old);
  minVersion = '0.0.1';

  // 9) site açılamazsa yerel "bağlanılamadı" sayfası
  const closed = http.createServer();
  await new Promise((r) => closed.listen(0, '127.0.0.1', r));
  const deadPort = closed.address().port;
  await new Promise((r) => closed.close(r));
  const offline = launch(`http://localhost:${deadPort}/`);
  await sleep(7000);
  const offlineTitle = documentName(offline.pid);
  check('site açılamazsa "bağlanılamadı" sayfası', /bağlanılamadı/i.test(offlineTitle), offlineTitle);
  await stop(offline);

  server.close();
  restoreSettings();
  const failed = results.filter((x) => !x.passed).length;
  console.log(`\n${results.length - failed}/${results.length} geçti`);
  process.exit(failed ? 1 : 0);
})().catch((e) => { restoreSettings(); console.error(e); process.exit(1); });
