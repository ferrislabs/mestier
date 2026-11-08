import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'
import { describe, expect, it } from 'vitest'

const SRC = join(__dirname, '..', '..', '..')
const HR = join(SRC, 'pages', 'hr')

const LOCAL_STORAGE_EXCEPTIONS: readonly string[] = []

function sourceFiles(dir: string): string[] {
	return readdirSync(dir).flatMap((name) => {
		const path = join(dir, name)
		if (statSync(path).isDirectory()) return sourceFiles(path)
		return /\.(ts|tsx)$/.test(name) && !/\.test\.tsx?$/.test(name) ? [path] : []
	})
}

function production(): { path: string; text: string }[] {
	return [HR, join(SRC, 'pages', 'rh-nav')]
		.flatMap(sourceFiles)
		.map((path) => ({
			path: relative(SRC, path),
			text: readFileSync(path, 'utf8'),
		}))
}

describe('RH mock layer', () => {
	it('spec_rh_9_1 no production file under pages/hr imports the mock layer or localStorage for backend data', () => {
		const files = production()
		expect(files.length).toBeGreaterThan(50)

		const mockImports = files
			.filter(({ text }) => /mocks\/rh-|rh-seed|rh-store|rh-binding/.test(text))
			.map(({ path }) => path)
		expect(mockImports).toEqual([])

		const storage = files
			.filter(({ text }) => /localStorage|sessionStorage/.test(text))
			.map(({ path }) => path)
			.filter((path) => !LOCAL_STORAGE_EXCEPTIONS.includes(path))
		expect(storage).toEqual([])
	})

	it('spec_rh_9_1 the mock modules are gone from the tree', () => {
		const everything = sourceFiles(SRC).map((path) => relative(SRC, path))
		expect(everything.filter((path) => /(^|\/)mocks\/rh-/.test(path))).toEqual(
			[],
		)
	})

	it('spec_rh_9_1 no RH production file imports from the test fixtures directory', () => {
		const offenders = production()
			.filter(({ text }) => /from '#\/test\//.test(text))
			.map(({ path }) => path)
		expect(offenders).toEqual([])
	})
})
