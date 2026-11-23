// window.nahrokMasaustu: sitenin uygulamadan isteyebileceği kısa liste (PLAN.md çizgi 3).
// Her sayfa yüklenişinde, sayfanın kendi betiklerinden önce çalışır. Yalnız Nahrok sitesinde tanımlanır.
// Komutların izni ayrıca Rust tarafında (capabilities/site.json + köken denetimi) verilir; bu dosya yalnız kolaylıktır.
(() => {
  if (location.origin !== '__NAHROK_KOKEN__') return;
  const cagir = (komut, args) => {
    const ic = window.__TAURI_INTERNALS__;
    return ic && typeof ic.invoke === 'function' ? ic.invoke(komut, args) : Promise.reject(new Error('köprü yok'));
  };
  const dinleyiciler = new Set();
  // Kısayola basılınca uygulama bunu çağırır ('sustur' | 'sagirlastir')
  Object.defineProperty(window, '__nahrokKisayol', {
    value: (eylem) => {
      if (eylem !== 'sustur' && eylem !== 'sagirlastir') return;
      for (const cb of dinleyiciler) { try { cb(eylem); } catch (_) { /* dinleyicinin hatası diğerlerini durdurmaz */ } }
    },
    writable: false, configurable: false,
  });
  const metin = (v) => (typeof v === 'string' ? v.slice(0, 64) : undefined);
  const kopru = Object.freeze({
    surum: '__NAHROK_SURUM__',
    platform: 'windows',
    rozet: (sayi) => { cagir('rozet', { sayi: Math.max(0, Math.min(99, Number(sayi) | 0)) }).catch(() => {}); },
    sesDurumu: (d) => {
      d = d || {};
      cagir('ses_durumu', { durum: { kanalda: !!d.kanalda, susturuldu: !!d.susturuldu, sagir: !!d.sagir } }).catch(() => {});
    },
    kisayolDinle: (cb) => {
      if (typeof cb !== 'function') return () => {};
      dinleyiciler.add(cb);
      return () => { dinleyiciler.delete(cb); };
    },
    kisayollar: (degisiklik) => cagir('kisayollar', {
      degisiklik: degisiklik ? { sustur: metin(degisiklik.sustur), sagirlastir: metin(degisiklik.sagirlastir) } : null,
    }),
  });
  Object.defineProperty(window, 'nahrokMasaustu', { value: kopru, writable: false, configurable: false });
})();
