import { defineConfig } from 'vite';

export default defineConfig({
  base: '/notes/',
  server: { port: 5184, strictPort: false },
  build: {
    target: 'es2022',
    outDir: 'docs',
    emptyOutDir: true,
  },
});
