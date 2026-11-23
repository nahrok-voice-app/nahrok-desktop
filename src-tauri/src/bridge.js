// window.nahrokDesktop: sitenin uygulamadan isteyebileceği kısa liste (PLAN.md çizgi 3).
// Her sayfa yüklenişinde, sayfanın kendi betiklerinden önce çalışır ve yalnız Nahrok sitesinde tanımlanır.
// Asıl izin Rust tarafındadır (capabilities/site.json + köken denetimi); bu dosya yalnız kolaylıktır.
(() => {
  if (location.origin !== '__NAHROK_SITE_ORIGIN__') return;

  const invoke = (command, args) => {
    const internals = window.__TAURI_INTERNALS__;
    return internals && typeof internals.invoke === 'function'
      ? internals.invoke(command, args)
      : Promise.reject(new Error('Nahrok köprüsü hazır değil'));
  };
  const ignore = () => {};

  // Kısayola basılınca (ya da tepsi menüsünden) uygulama bunu çağırır
  const ACTIONS = ['toggleMute', 'toggleDeafen'];
  const listeners = new Set();
  Object.defineProperty(window, '__nahrokShortcut', {
    value: (action) => {
      if (!ACTIONS.includes(action)) return;
      for (const listener of listeners) {
        try { listener(action); } catch { /* bir dinleyicinin hatası diğerlerini durdurmaz */ }
      }
    },
    writable: false,
    configurable: false,
  });

  const api = Object.freeze({
    version: '__NAHROK_VERSION__',
    platform: 'windows',
    // Okunmamış metin kanalı sayısı (0 rozeti kaldırır)
    setBadge: (count) => {
      invoke('set_badge', { count: Math.max(0, Math.min(99, Number(count) | 0)) }).catch(ignore);
    },
    // { inVoice, muted, deafened }
    setVoiceState: (state) => {
      const s = state || {};
      invoke('set_voice_state', { state: { inVoice: !!s.inVoice, muted: !!s.muted, deafened: !!s.deafened } }).catch(ignore);
    },
    // Dinleyici 'toggleMute' | 'toggleDeafen' alır; dönen işlev dinlemeyi bırakır
    onShortcut: (listener) => {
      if (typeof listener !== 'function') return ignore;
      listeners.add(listener);
      return () => { listeners.delete(listener); };
    },
    openSettings: () => { invoke('open_settings').catch(ignore); },
  });
  Object.defineProperty(window, 'nahrokDesktop', { value: api, writable: false, configurable: false });

  // Ctrl+, ayarları açar (masaüstü uygulamalarındaki alışılmış kısayol)
  window.addEventListener('keydown', (e) => {
    if (e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey && e.key === ',') {
      e.preventDefault();
      api.openSettings();
    }
  }, true);
})();
