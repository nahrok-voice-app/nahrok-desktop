// Site açılamayınca gösterilir. Site adresi uygulamadan gelir; uygulama başka adrese gezinmeye zaten izin vermez.
const param = new URLSearchParams(location.search).get('site') || '';
const site = /^https?:\/\//.test(param) ? param : 'https://ekorhan.site/';

function retry() {
  document.getElementById('status').textContent = 'Deneniyor…';
  location.href = site;
}

document.getElementById('retry').addEventListener('click', retry);
window.addEventListener('online', retry);
setTimeout(retry, 15000);
