import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

(() => {
  if (!window.__TAURI_INTERNALS__) return;

  const show = (html, buttons) => {
    const modal = document.querySelector('#modal');
    if (!modal) return;
    document.querySelector('#modal-body').innerHTML = html;
    const actions = document.querySelector('#modal-btns');
    actions.replaceChildren();
    for (const item of buttons) {
      const button = document.createElement('button');
      button.className = item.primary ? 'btn btn-primary' : 'btn btn-outline';
      button.textContent = item.label;
      button.addEventListener('click', item.action, { once: true });
      actions.append(button);
    }
    modal.classList.remove('hidden');
  };

  window.addEventListener('DOMContentLoaded', async () => {
    try {
      const update = await check();
      if (!update) return;

      show(
        `<h2>Update tersedia</h2><p>Versi ${update.version} siap dipasang. Update akan diunduh dan aplikasi dimulai ulang.</p>`,
        [
          { label: 'Nanti', action: () => document.querySelector('#modal').classList.add('hidden') },
          {
            label: 'Update sekarang',
            primary: true,
            action: async () => {
              const actions = document.querySelector('#modal-btns');
              actions.replaceChildren();
              const progress = document.createElement('p');
              progress.textContent = 'Mengunduh update…';
              actions.append(progress);
              try {
                await update.downloadAndInstall();
                await relaunch();
              } catch (error) {
                progress.textContent = `Gagal memasang update: ${error?.message || error}`;
                const close = document.createElement('button');
                close.className = 'btn btn-outline';
                close.textContent = 'Tutup';
                close.addEventListener('click', () => document.querySelector('#modal').classList.add('hidden'));
                actions.append(close);
              }
            },
          },
        ],
      );
    } catch (error) {
      console.warn('Pemeriksaan update gagal:', error);
    }
  }, { once: true });
})();