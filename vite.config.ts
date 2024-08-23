/// <reference types="vitest" />
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';
import AutoImport from 'unplugin-auto-import/vite';
import { defineConfig, type UserConfig } from 'vite';
import vuetify, { transformAssetUrls } from 'vite-plugin-vuetify';
import { visualizer } from 'rollup-plugin-visualizer';
import { VueRouterAutoImports } from 'unplugin-vue-router';
import VueRouter from 'unplugin-vue-router/vite';
import { fileURLToPath, URL } from 'url';

// https://vitejs.dev/config/
export default defineConfig(({ command, mode }): UserConfig => {
	const config = {
		plugins: [
			VueRouter({
				routesFolder: 'src/pages',
				dts: './src/typed-router.d.ts'
			}),
			vue(),
			vuetify({
				autoImport: true
				// styles: { configFile: 'src/styles/settings.scss' },
			}),
			AutoImport({
				imports: ['vue', VueRouterAutoImports],
				// imports: ['vue'],
				dts: './src/auto-imports.d.ts',
				eslintrc: {
					enabled: true,
					filepath: resolve(__dirname, '.eslintrc-auto-import.json')
				}
			})
		],
		clearScreen: false,
		envPrefix: ['VITE_', 'TAURI_'],
		server: {
			port: 5173,
			strictPort: true
		},
		esbuild: {
			supported: {
				'top-level-await': true
			}
		},
		build: {
			outDir: './dist',
			// See https://tauri.app/v1/references/webview-versions for details
			target: ['es2021', 'chrome100', 'safari14'],
			// target: ['esnext'],
			minify: !!!process.env.TAURI_DEBUG,
			sourcemap: !!process.env.TAURI_DEBUG,
			emptyOutDir: true,
			rollupOptions: {
				input: {
					main: resolve(__dirname, 'index.html'),
					installer: resolve(__dirname, 'src/installer/index.html')
				}
				// 	output: {
				// 		manualChunks: {
				// 			// Split external library from transpiled code.
				// 			vue: ['vue', 'vue-router', 'pinia', 'pinia-plugin-persistedstate'],
				// 			vuetify: [
				// 				'vuetify',
				// 				'vuetify/components',
				// 				'vuetify/directives',
				// 				// 'vuetify/lib/labs',
				// 				'webfontloader',
				// 			],
				// 			materialdesignicons: ['@mdi/font/css/materialdesignicons.css'],
				// 		},
				// 		plugins: [
				// 			mode === 'analyze'
				// 				? // rollup-plugin-visualizer
				// 				// https://github.com/btd/rollup-plugin-visualizer
				// 				visualizer({
				// 					open: true,
				// 					filename: 'dist/stats.html',
				// 				})
				// 				: undefined,
				// 		],
				// 	},
			}
		},
		test: {
			include: ['tests/unit/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}']
		},
		resolve: {
			alias: [{ find: '@', replacement: fileURLToPath(new URL('./src', import.meta.url)) }]
		}
	};

	return config;
});
