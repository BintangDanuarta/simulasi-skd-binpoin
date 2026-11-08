import { NextResponse } from 'next/server';

const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export async function POST(request: Request) {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Format permintaan tidak valid.' }, { status: 400 });
  }

  const email = typeof body === 'object' && body !== null && 'email' in body
    ? String((body as { email: unknown }).email).trim().toLowerCase()
    : '';

  if (email.length > 254 || !emailPattern.test(email)) {
    return NextResponse.json({ error: 'Masukkan alamat email yang valid.' }, { status: 400 });
  }

  // TODO(production): persist the invite and send the download link with a transactional
  // email provider. Do not claim the app has been emailed until that provider succeeds.
  console.info(`[beta-invite] signup received: ${email}`);
  return NextResponse.json({
    ok: true,
    message: 'Pendaftaran beta tercatat. Pengiriman undangan email belum diaktifkan.',
  }, { status: 202 });
}
