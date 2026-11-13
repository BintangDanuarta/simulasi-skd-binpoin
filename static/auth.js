'use strict';

const form = document.querySelector('#auth-form');
const button = document.querySelector('#submit');
const errorEl = document.querySelector('#error');
const registerMode = location.pathname === '/register' || location.pathname.endsWith('/register');
const appBase = location.pathname === '/app' || location.pathname.startsWith('/app/') ? '/app' : '';

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  button.disabled = true;
  errorEl.classList.add('hidden');
  const email = document.querySelector('#email').value.trim();
  const password = document.querySelector('#password').value;
  try {
    const response = await fetch(`${appBase}${registerMode ? '/api/auth/register' : '/api/auth/login'}`, {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });
    const data = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(data.error || 'Tidak dapat memproses akun.');
    location.replace(`${appBase}/`);
  } catch (error) {
    errorEl.textContent = error.message || 'Terjadi kesalahan jaringan.';
    errorEl.classList.remove('hidden');
    button.disabled = false;
  }
});