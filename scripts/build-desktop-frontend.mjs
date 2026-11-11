import { build } from 'esbuild';

await build({
  entryPoints: ['scripts/desktop-update.js'],
  bundle: true,
  format: 'iife',
  platform: 'browser',
  target: ['safari14'],
  outfile: 'static/desktop-update.bundle.js',
});
