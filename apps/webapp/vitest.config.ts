import { fileURLToPath } from 'node:url'
import viteReact from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'

export default defineConfig({
	plugins: [viteReact()],
	resolve: {
		alias: [
			{
				find: /^#\//,
				replacement: `${fileURLToPath(new URL('./src', import.meta.url))}/`,
			},
		],
	},
	test: {
		environment: 'jsdom',
		setupFiles: ['./vitest.setup.ts'],
		include: ['src/**/*.test.{ts,tsx}'],
	},
})
