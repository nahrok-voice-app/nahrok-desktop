// Site, uygulamanın bu sürümünden daha yenisini isteyince gösterilir (desktop.json → minVersion).
const query = new URLSearchParams(location.search);
const store = query.get('store') || '';
if (store.startsWith('ms-windows-store://')) document.getElementById('store').href = store;
const version = (query.get('version') || '').replace(/[^0-9.]/g, '');
if (version) document.getElementById('version').textContent = `Yüklü sürüm: ${version}`;
