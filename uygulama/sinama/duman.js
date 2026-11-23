// Duman testi (PLAN.md Sıra 2): hata ayıklama derlemesini yerel bir sahte siteye karşı çalıştırır, çizgi 2-4'ü ve v1
// davranışlarını sınar. Önce: npm run derle:sinama   Sonra: npm run duman
// Ekranda kısa süre Nahrok penceresi açılıp kapanır; Ctrl+Shift+F11 tuşları bir kez gönderilir.
const http = require('http');
const path = require('path');
const { spawn, execFileSync } = require('child_process');

const EXE = process.argv[2] || path.join(__dirname, '..', 'src-tauri', 'target', 'debug', 'nahrok.exe');
const fs = require('fs');
const AYAR = path.join(process.env.APPDATA || '', 'com.nahrok.masaustu', 'kisayollar.json');
const eskiAyar = fs.existsSync(AYAR) ? fs.readFileSync(AYAR) : null;
const ayariGeriKoy = () => { try { eskiAyar ? fs.writeFileSync(AYAR, eskiAyar) : fs.rmSync(AYAR, { force: true }); } catch {} };
const bekle = (ms) => new Promise((r) => setTimeout(r, ms));
const sonuclar = [];
const kontrol = (ad, gecti, ayrinti = '') => { sonuclar.push({ ad, gecti: !!gecti }); console.log(`${gecti ? '✔' : '✖'} ${ad}${ayrinti ? `  (${ayrinti})` : ''}`); };

function ps(betik) {
  return execFileSync('powershell.exe', ['-NoProfile', '-Command', betik], { encoding: 'utf8' }).trim();
}
// Ana pencere "Tauri Window" sınıfındaki penceredir. MainWindowHandle kullanılmaz: ana pencere gizlenince .NET onun
// yerine tek kopya eklentisinin görünmez ileti penceresini döndürür.
const pencereGorunur = (pid) => ps(`Add-Type -Namespace W -Name U -MemberDefinition @'
public delegate bool CB(System.IntPtr h, System.IntPtr l);
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool EnumWindows(CB cb, System.IntPtr l);
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(System.IntPtr h, out uint p);
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool IsWindowVisible(System.IntPtr h);
[System.Runtime.InteropServices.DllImport("user32.dll", CharSet=System.Runtime.InteropServices.CharSet.Unicode)] public static extern int GetClassName(System.IntPtr h, System.Text.StringBuilder s, int n);
'@
$script:g = 'yok'
[W.U]::EnumWindows({ param($h,$l) $p=0; [void][W.U]::GetWindowThreadProcessId($h,[ref]$p); $c=New-Object Text.StringBuilder 64; [void][W.U]::GetClassName($h,$c,64)
  if ($p -eq ${pid} -and $c.ToString() -eq 'Tauri Window') { $script:g = [W.U]::IsWindowVisible($h); return $false }; $true }, [IntPtr]::Zero) | Out-Null
$script:g`);
const calisiyor = (pid) => { try { process.kill(pid, 0); return true; } catch { return false; } };
const kapatIste = (pid) => ps(`$p=Get-Process -Id ${pid} -ErrorAction SilentlyContinue; if ($p) { $p.CloseMainWindow() | Out-Null }`);
const belgeAdi = (pid) => ps(`Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes; $A=[Windows.Automation.AutomationElement];
  $w=$A::RootElement.FindFirst([Windows.Automation.TreeScope]::Children,(New-Object Windows.Automation.PropertyCondition($A::ProcessIdProperty, ${pid})));
  if ($w) { $d=$w.FindFirst([Windows.Automation.TreeScope]::Descendants,(New-Object Windows.Automation.PropertyCondition($A::ControlTypeProperty,[Windows.Automation.ControlType]::Document))); if ($d) { $d.Current.Name } }`);

// Sahte site: her yüklenişte sınamaları koşar, sonuçları /olay'a yollar
const SAYFA = `<!doctype html><meta charset="utf-8"><title>Duman</title><body>duman<script>
const yolla = (tur, veri) => fetch('/olay', { method: 'POST', body: JSON.stringify({ tur, veri }) });
const k = window.nahrokMasaustu;
(async () => {
  const r = { kopru: !!k, donmus: !!k && Object.isFrozen(k), surum: k && k.surum };
  try { k.rozet(3); r.rozet = true; } catch (e) { r.rozet = String(e); }
  r.kisayolIlk = await k.kisayollar().catch(e => ({ hata: String(e) }));
  r.kisayolYeni = await k.kisayollar({ sustur: 'Ctrl+Shift+F11' }).catch(e => ({ hata: String(e) }));
  r.kisayolGecersiz = await k.kisayollar({ sagirlastir: 'Ctrl+Shift+Bilinmeyen' }).catch(e => ({ hata: String(e) }));
  const dene = async (komut, args) => { try { await window.__TAURI_INTERNALS__.invoke(komut, args); return 'izin verildi'; } catch (e) { return 'reddedildi'; } };
  r.openerReddi = await dene('plugin:opener|open_url', { url: 'https://example.com' });
  r.kisayolEklentiReddi = await dene('plugin:global-shortcut|unregister_all', {});
  r.pencereReddi = await dene('plugin:window|close', {});
  r.kamera = await navigator.mediaDevices.getUserMedia({ video: true }).then(s => { s.getTracks().forEach(t => t.stop()); return 'verildi'; }, e => e.name);
  r.mikrofon = await navigator.mediaDevices.getUserMedia({ audio: true }).then(s => { s.getTracks().forEach(t => t.stop()); return 'verildi'; }, e => e.name);
  r.yeniPencere = window.open('about:blank') === null ? 'engellendi' : 'açıldı';
  if (location.search.includes('kanalda')) k.sesDurumu({ kanalda: true });
  k.kisayolDinle(e => yolla('kisayol', e));
  setInterval(() => yolla('nabiz', document.visibilityState), 500);
  yolla('sonuc', r);
  setTimeout(() => { location.href = 'file:///C:/Windows/win.ini'; }, 300);   // başka adrese gezinme engellenmeli
})();
</script>`;

let olaylar = [];
let enAz = '0.0.1';
const sunucu = http.createServer((req, res) => {
  if (req.method === 'POST' && req.url === '/olay') {
    let s = ''; req.on('data', (d) => (s += d)); req.on('end', () => { olaylar.push({ ...JSON.parse(s), zaman: Date.now() }); res.end(); });
    return;
  }
  if (req.url === '/masaustu.json') { res.setHeader('Content-Type', 'application/json'); res.end(JSON.stringify({ enAzSurum: enAz })); return; }
  res.setHeader('Content-Type', 'text/html; charset=utf-8'); res.end(SAYFA);
});

const ac = (url) => spawn(EXE, [], { stdio: 'ignore', env: { ...process.env, NAHROK_URL: url } });
const olayBekle = async (tur, ms = 20000, sonra = 0) => {
  for (let t = 0; t < ms; t += 200) { const o = olaylar.find((x) => x.tur === tur && x.zaman > sonra); if (o) return o; await bekle(200); }
  return null;
};

(async () => {
  await new Promise((r) => sunucu.listen(0, '127.0.0.1', r));
  const port = sunucu.address().port;
  const site = `http://localhost:${port}/`;

  // 1) köprü, izinler, gezinme
  const uyg = ac(site);
  const s = await olayBekle('sonuc');
  if (!s) { kontrol('site yüklendi ve köprü yanıt verdi', false); try { process.kill(uyg.pid); } catch {} ayariGeriKoy(); process.exit(1); }
  const r = s.veri;
  kontrol('köprü var, donmuş, sürüm var', r.kopru && r.donmus && /^\d+\.\d+\.\d+$/.test(r.surum), r.surum);
  kontrol('rozet çağrısı', r.rozet === true);
  kontrol('kısayollar okunur', r.kisayolIlk && r.kisayolIlk.sustur, JSON.stringify(r.kisayolIlk));
  kontrol('kısayol değiştirilir', r.kisayolYeni && r.kisayolYeni.sustur === 'Ctrl+Shift+F11' && !r.kisayolYeni.hata);
  kontrol('geçersiz kısayol reddedilir', r.kisayolGecersiz && r.kisayolGecersiz.hata);
  kontrol('liste dışı komut: opener reddedilir', r.openerReddi === 'reddedildi');
  kontrol('liste dışı komut: kısayol eklentisi reddedilir', r.kisayolEklentiReddi === 'reddedildi');
  kontrol('liste dışı komut: pencere kapatma reddedilir', r.pencereReddi === 'reddedildi');
  kontrol('kamera izni verilmez', r.kamera !== 'verildi', r.kamera);
  kontrol('mikrofon izni verilir', r.mikrofon === 'verildi' || r.mikrofon === 'NotFoundError', r.mikrofon);
  kontrol('yeni pencere açılmaz', r.yeniPencere === 'engellendi');
  await bekle(1500);
  const yuklenme = olaylar.filter((o) => o.tur === 'sonuc').length;
  kontrol('başka adrese (file:) gezinme engellenir', yuklenme === 1 && (await olayBekle('nabiz', 3000, Date.now() - 400)));

  // 2) genel kısayol (RegisterHotKey): Ctrl+Shift+F11
  ps(`(New-Object -ComObject WScript.Shell).SendKeys('^+{F11}')`);
  const kis = await olayBekle('kisayol', 5000);
  kontrol('genel kısayol siteye ulaşır', kis && kis.veri === 'sustur', kis && kis.veri);

  // 3) tek kopya
  const ikinci = ac(site);
  await bekle(4000);
  kontrol('ikinci açılış kapanır, ilki sürer', ikinci.exitCode !== null && calisiyor(uyg.pid), `ikinci çıkış=${ikinci.exitCode}`);

  // 4) sesli kanalda değilken kapat = çık; seste iken kapat = tepsiye in (süreç ve sayfa sürer)
  kapatIste(uyg.pid);
  await bekle(2500);
  kontrol('sesli kanalda değilken pencereyi kapatmak uygulamayı kapatır', !calisiyor(uyg.pid));

  olaylar = [];
  const uyg2 = ac(`${site}?kanalda`);   // sayfa ?kanalda görünce sesDurumu({ kanalda: true }) bildirir
  await olayBekle('sonuc');
  await bekle(800);
  kapatIste(uyg2.pid);
  await bekle(2000);
  const gorunur = pencereGorunur(uyg2.pid);
  const kapanis = Date.now();
  await bekle(2500);
  const nabizSurdu = olaylar.some((o) => o.tur === 'nabiz' && o.zaman > kapanis);
  kontrol('seste iken kapatınca tepsiye iner (süreç sürer, pencere gizli, sayfa çalışır)',
    calisiyor(uyg2.pid) && gorunur === 'False' && nabizSurdu, `görünür=${gorunur} nabız=${nabizSurdu}`);
  try { process.kill(uyg2.pid); } catch {}
  await bekle(1000);

  // 5) en düşük sürüm: site 99.0.0 isterse "güncelle" sayfası
  enAz = '99.0.0';
  const uyg3 = ac(site);
  await bekle(7000);
  const ad3 = belgeAdi(uyg3.pid);
  kontrol('eski sürüm "Store\'dan güncelle" sayfasını görür', /güncelleme gerekli/i.test(ad3), ad3);
  try { process.kill(uyg3.pid); } catch {}
  enAz = '0.0.1';
  await bekle(1000);

  // 6) site açılamazsa yerel "bağlanılamadı" sayfası
  const kapali = http.createServer(); await new Promise((r) => kapali.listen(0, '127.0.0.1', r));
  const olu = kapali.address().port; await new Promise((r) => kapali.close(r));
  const uyg4 = ac(`http://localhost:${olu}/`);
  await bekle(7000);
  const ad4 = belgeAdi(uyg4.pid);
  kontrol('site açılamazsa "bağlanılamadı" sayfası', /bağlanılamadı/i.test(ad4), ad4);
  try { process.kill(uyg4.pid); } catch {}

  sunucu.close();
  ayariGeriKoy();
  const basarisiz = sonuclar.filter((x) => !x.gecti).length;
  console.log(`\n${sonuclar.length - basarisiz}/${sonuclar.length} geçti`);
  process.exit(basarisiz ? 1 : 0);
})().catch((e) => { ayariGeriKoy(); console.error(e); process.exit(1); });
