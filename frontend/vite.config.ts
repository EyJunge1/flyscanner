import adapter from '@sveltejs/adapter-static';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			adapter: adapter({
				fallback: 'index.html',
				precompress: false
			}),
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			}
		})
	],
	server: {
		proxy: {
			'/api': 'http://127.0.0.1:8080',
			'/ws': { target: 'ws://127.0.0.1:8080', ws: true },
			'/metrics': 'http://127.0.0.1:8080'
		}
	}
});
