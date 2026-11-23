// Ayarlar penceresi. Uygulamanın kendi yerel sayfasıdır; buradaki komutlar yalnız bu pencereye açıktır
// (src-tauri/capabilities/settings.json), site bunları çağıramaz.
const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
const byId = (id) => document.getElementById(id);

// Windows'un başlangıç görevi durumuna göre anahtarın altında çıkan not
const STARTUP_NOTES = {
  unavailable: { text: 'Bu ayar yalnız kurulu sürümde çalışır.' },
  disabledByUser: {
    text: "Başlangıçta açılmayı Windows ayarlarından kapatmışsın. Yeniden açmak için Windows ayarlarında Nahrok'u aç.",
    windowsButton: true,
  },
  disabledByPolicy: { text: 'Bu ayarı bilgisayarın yöneticisi kapatmış.' },
  enabledByPolicy: { text: 'Bu ayarı bilgisayarın yöneticisi açmış.' },
};

let settings = null;

const setSwitch = (el, on, enabled = true) => {
  el.setAttribute('aria-checked', on ? 'true' : 'false');
  el.disabled = !enabled;
};

function render() {
  const startup = settings.startAtLogin;
  const startupOn = startup === 'enabled' || startup === 'enabledByPolicy';
  setSwitch(byId('start-at-login'), startupOn, startup === 'enabled' || startup === 'disabled');
  setSwitch(byId('start-hidden'), settings.startHidden, startupOn);
  setSwitch(byId('close-to-tray'), settings.closeToTray);

  const note = STARTUP_NOTES[startup];
  byId('startup-note').hidden = !note;
  byId('startup-note-text').textContent = note ? note.text : '';
  byId('open-windows-startup').hidden = !(note && note.windowsButton);

  for (const box of document.querySelectorAll('.key-box')) {
    if (box.classList.contains('is-listening')) continue;
    const value = settings[box.dataset.action];
    box.textContent = value || 'Yok';
    box.setAttribute('aria-label', `Kısayol: ${value || 'yok'}. Değiştirmek için bas.`);
    box.disabled = false;
  }
  byId('version').textContent = `Nahrok ${settings.version}`;
}

function showShortcutError(message) {
  const el = byId('shortcut-error');
  el.textContent = message || '';
  el.hidden = !message;
}

async function update(patch) {
  const result = await invoke('update_settings', { patch });
  settings = result.settings;
  showShortcutError(result.error);
  render();
}

byId('start-at-login').addEventListener('click', async (e) => {
  const enable = e.currentTarget.getAttribute('aria-checked') !== 'true';
  e.currentTarget.disabled = true;
  settings.startAtLogin = await invoke('set_start_at_login', { enabled: enable });
  render();
});
byId('start-hidden').addEventListener('click', (e) => update({ startHidden: e.currentTarget.getAttribute('aria-checked') !== 'true' }));
byId('close-to-tray').addEventListener('click', (e) => update({ closeToTray: e.currentTarget.getAttribute('aria-checked') !== 'true' }));
byId('open-windows-startup').addEventListener('click', () => invoke('open_windows_startup_settings'));

// ---------------------------------------------------------------- kısayol kaydı

// KeyboardEvent.code → kısayol metni ("Ctrl+Shift+M"). Uygulama aynı metni çözümler (global-hotkey).
const NAMED_KEYS = new Set(['Backquote', 'Minus', 'Equal', 'BracketLeft', 'BracketRight', 'Backslash', 'Semicolon',
  'Quote', 'Comma', 'Period', 'Slash', 'Pause', 'ScrollLock', 'Insert', 'Home', 'End', 'PageUp', 'PageDown',
  'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Space']);
// Tek başına basılınca yazıyı bozmayan tuşlar: değiştiricisiz de olabilir
const STANDALONE = /^(F([1-9]|1[0-9]|2[0-4])|Pause|ScrollLock|Insert)$/;

function shortcutFromEvent(e) {
  const code = e.code || '';
  let key = null;
  if (/^Key[A-Z]$/.test(code)) key = code.slice(3);
  else if (/^Digit[0-9]$/.test(code)) key = code.slice(5);
  else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) key = code;
  else if (/^Numpad[0-9]$/.test(code)) key = 'Num' + code.slice(6);
  else if (NAMED_KEYS.has(code)) key = code;
  if (!key) return null;
  const modifiers = [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift'].filter(Boolean);
  if (!modifiers.length && !STANDALONE.test(key)) return null;
  return [...modifiers, key].join('+');
}

function stopListening(box) {
  box.classList.remove('is-listening');
  render();
}

for (const box of document.querySelectorAll('.key-box')) {
  box.addEventListener('click', () => {
    if (box.classList.contains('is-listening')) { stopListening(box); return; }
    box.classList.add('is-listening');
    box.textContent = 'Tuşlara bas…';
  });
  box.addEventListener('blur', () => { if (box.classList.contains('is-listening')) stopListening(box); });
  box.addEventListener('keydown', (e) => {
    if (!box.classList.contains('is-listening')) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape') { stopListening(box); return; }
    let value = null;
    if (e.key === 'Backspace' || e.key === 'Delete') value = '';
    else value = shortcutFromEvent(e);
    if (value === null) return;   // yalnız değiştirici basıldı: asıl tuşu bekle
    box.classList.remove('is-listening');
    update({ [box.dataset.action]: value });
  });
}

invoke('get_settings').then((s) => { settings = s; render(); });
