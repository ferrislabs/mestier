import { describe, expect, it } from 'vitest'
import {
	buildConnectorExpression,
	flattenExamplePaths,
} from '#/pages/automation/lib/expression'

describe('flattenExamplePaths', () => {
	it('always includes the whole-output path first', () => {
		const paths = flattenExamplePaths({ total: 42 })
		expect(paths[0]).toEqual({ path: '', preview: '{…}' })
	})

	it('walks nested object keys with dot paths', () => {
		const paths = flattenExamplePaths({ quote: { total: 42 } })
		expect(paths.map((p) => p.path)).toContain('quote.total')
		expect(paths.find((p) => p.path === 'quote.total')?.preview).toBe('42')
	})

	it('samples only the first array element, with bracket indices', () => {
		const paths = flattenExamplePaths({
			lines: [{ total: 10 }, { total: 20 }],
		})
		expect(paths.map((p) => p.path)).toContain('lines[0].total')
		expect(paths.map((p) => p.path)).not.toContain('lines[1].total')
	})

	it('stops descending past the depth cap', () => {
		const deeplyNested = { a: { b: { c: { d: { e: 'too deep' } } } } }
		const paths = flattenExamplePaths(deeplyNested)
		expect(paths.map((p) => p.path)).not.toContain('a.b.c.d.e')
	})

	it('renders a short preview per JSON type', () => {
		const paths = flattenExamplePaths({
			s: 'hi',
			n: 1,
			b: true,
			nul: null,
			arr: [1, 2, 3],
		})
		const preview = (path: string) =>
			paths.find((p) => p.path === path)?.preview

		expect(preview('s')).toBe('"hi"')
		expect(preview('n')).toBe('1')
		expect(preview('b')).toBe('true')
		expect(preview('nul')).toBe('null')
		expect(preview('arr')).toBe('[3]')
	})
})

describe('buildConnectorExpression', () => {
	it('references the whole output with no path', () => {
		expect(buildConnectorExpression('c1', '')).toBe(
			'{{ connectors.c1.output }}',
		)
	})

	it('appends a dotted path', () => {
		expect(buildConnectorExpression('c1', 'quote.total')).toBe(
			'{{ connectors.c1.output.quote.total }}',
		)
	})

	it('appends an array-index path without an extra dot', () => {
		expect(buildConnectorExpression('c1', '[0].total')).toBe(
			'{{ connectors.c1.output[0].total }}',
		)
	})
})
