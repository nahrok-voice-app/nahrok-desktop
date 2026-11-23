// Nahrok masaüstü, Electron denemesi. PLAN.md çizgi 1-4'e göre: siteyi yükler, web'e güvenmez.
const { app, BrowserWindow, Menu, desktopCapturer, session, shell } = require('electron');

// Tek sabit adres. Denemede ortam değişkeniyle verilir: NAHROK_URL=https://... npm start
const SITE_URL = process.env.NAHROK_URL || 'http://localhost:5173';
const SITE_ORIGIN = new URL(SITE_URL).origin;

// Çizgi 2: izin olarak yalnız mikrofon, ekran yakalama ve tam ekran.
const IZINLI = new Set(['media', 'display-capture', 'fullscreen']);

function siteMi(url) {
  try { return new URL(url).origin === SITE_ORIGIN; } catch { return false; }
}

if (!app.requestSingleInstanceLock()) app.quit(); // tek kopya çalışma

let pencere;

// Çizgi 4: kaynağı kullanıcı uygulamanın kendi menüsünde seçer, web kodu seçemez.
function kaynakSec(kaynaklar) {
  return new Promise((resolve) => {
    let secildi = false;
    const menu = Menu.buildFromTemplate([
      { label: 'Paylaşılacak ekranı seç', enabled: false },
      { type: 'separator' },
      ...kaynaklar.map((k) => ({
        label: k.name,
        click: () => { secildi = true; resolve(k); },
      })),
      { type: 'separator' },
      { label: 'Vazgeç', click: () => { secildi = true; resolve(null); } },
    ]);
    menu.popup({ window: pencere, callback: () => { if (!secildi) resolve(null); } });
  });
}

function pencereAc() {
  pencere = new BrowserWindow({
    width: 1280,
    height: 800,
    webPreferences: {
      sandbox: true,
      contextIsolation: true,
      nodeIntegration: false,
      webSecurity: true,
    },
  });

  // Başka adrese gezinme engellenir, dış bağlantılar sistem tarayıcısında açılır.
  pencere.webContents.on('will-navigate', (e, url) => {
    if (!siteMi(url)) {
      e.preventDefault();
      if (/^https?:/i.test(url)) shell.openExternal(url);
    }
  });
  pencere.webContents.setWindowOpenHandler(({ url }) => {
    if (/^https?:/i.test(url)) shell.openExternal(url);
    return { action: 'deny' };
  });

  pencere.loadURL(SITE_URL);
}

app.whenReady().then(() => {
  const oturum = session.defaultSession;

  oturum.setPermissionRequestHandler((wc, izin, cb) => cb(IZINLI.has(izin) && siteMi(wc.getURL())));
  oturum.setPermissionCheckHandler((wc, izin, origin) => IZINLI.has(izin) && siteMi(origin));

  // getDisplayMedia isteği: kullanıcı seçer, sistem sesi 'loopback' ile eklenir.
  // Denemenin ana sorusu: kanal konuşmaları yayına geri karışıyor mu? (restrictOwnAudio)
  oturum.setDisplayMediaRequestHandler(async (istek, cb) => {
    if (!siteMi(istek.frame.url)) return cb({});
    const kaynaklar = await desktopCapturer.getSources({ types: ['screen', 'window'] });
    const secilen = await kaynakSec(kaynaklar);
    if (!secilen) return cb({});
    cb({ video: secilen, audio: 'loopback' });
  });

  pencereAc();
});

app.on('second-instance', () => {
  if (pencere) { if (pencere.isMinimized()) pencere.restore(); pencere.focus(); }
});
app.on('window-all-closed', () => app.quit());
