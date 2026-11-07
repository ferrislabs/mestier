import { describe, expect, it } from 'vitest'
import {
	addDecoration,
	appliesToPage,
	clampRect,
	createDecoration,
	type Decoration,
	decorationLabel,
	duplicateDecoration,
	MAX_DECORATIONS,
	mmToPx,
	moveRect,
	nudgeRect,
	pxToMm,
	type Rect,
	removeDecoration,
	reorderDecoration,
	reorderedIndex,
	resizeRect,
	validateDecoration,
	validateDecorations,
} from '#/pages/doe-style/lib/decorations'

const rect = (
	over: Partial<
		Rect & { opacity: number; pages: 'cover' | 'inner' | 'all' }
	> = {},
): Decoration => ({
	kind: 'rectangle',
	fill: '#112233',
	x: 10,
	y: 10,
	width: 50,
	height: 20,
	...over,
})

describe('creation', () => {
	it('creates elements inside the page, an image waiting for its file', () => {
		for (const kind of ['image', 'rectangle', 'line'] as const) {
			const created = createDecoration(kind, '#0f3d36')
			expect(created.x + created.width).toBeLessThanOrEqual(210)
			expect(created.y + created.height).toBeLessThanOrEqual(297)
		}
		expect(validateDecoration(createDecoration('image', '#000000'))).toMatch(
			/image/,
		)
		expect(
			validateDecoration(createDecoration('rectangle', '#000000')),
		).toBeUndefined()
	})

	it('a line is a thin rectangle', () => {
		const line = createDecoration('line', '#0f3d36')
		expect(line.kind).toBe('rectangle')
		expect(decorationLabel(line)).toBe('Ligne')
		expect(validateDecoration(line)).toBeUndefined()
	})

	it('centers the rectangle', () => {
		const created = createDecoration('rectangle', '#0f3d36')
		expect(created.x + created.width / 2).toBe(105)
		expect(created.y + created.height / 2).toBe(148.5)
	})

	it('refuses to go beyond the maximum', () => {
		let list: Decoration[] = []
		for (let i = 0; i < MAX_DECORATIONS + 3; i++) {
			list = addDecoration(list, 'rectangle', '#000000')
		}
		expect(list).toHaveLength(MAX_DECORATIONS)
	})
})

describe('list operations', () => {
	const list = [rect({ x: 1 }), rect({ x: 2 }), rect({ x: 3 })]

	it('removes by index', () => {
		expect(removeDecoration(list, 1).map((d) => d.x)).toEqual([1, 3])
		expect(removeDecoration(list, 9)).toBe(list)
	})

	it('duplicates right after the source, shifted by 5 mm', () => {
		const next = duplicateDecoration(list, 0)
		expect(next).toHaveLength(4)
		expect(next[1]).toMatchObject({ x: 6, y: 15 })
		expect(next[0]).toBe(list[0])
	})

	it('keeps a duplicate inside the page', () => {
		const edge = [rect({ x: 160, y: 277, width: 50, height: 20 })]
		const next = duplicateDecoration(edge, 0)
		expect(next[1]).toMatchObject({ x: 160, y: 277 })
	})

	it('does not duplicate past the maximum', () => {
		const full = Array.from({ length: MAX_DECORATIONS }, () => rect())
		expect(duplicateDecoration(full, 0)).toBe(full)
	})

	it('reorders and clamps at the ends', () => {
		expect(reorderDecoration(list, 0, 'forward').map((d) => d.x)).toEqual([
			2, 1, 3,
		])
		expect(reorderDecoration(list, 2, 'backward').map((d) => d.x)).toEqual([
			1, 3, 2,
		])
		expect(reorderDecoration(list, 2, 'forward')).toBe(list)
		expect(reorderDecoration(list, 0, 'backward')).toBe(list)
		expect(reorderedIndex(list, 0, 'forward')).toBe(1)
		expect(reorderedIndex(list, 0, 'backward')).toBe(0)
	})
})

describe('pixels and millimeters', () => {
	it('converts both ways against the displayed width', () => {
		expect(pxToMm(210, 420)).toBe(105)
		expect(mmToPx(105, 420)).toBe(210)
		expect(pxToMm(10, 0)).toBe(0)
	})
})

describe('geometry', () => {
	const base = { x: 10, y: 10, width: 50, height: 20 }

	it('keeps a moved rectangle in the page', () => {
		expect(moveRect(base, -50, -50)).toMatchObject({ x: 0, y: 0 })
		expect(moveRect(base, 500, 500)).toMatchObject({ x: 160, y: 277 })
		expect(moveRect(base, 5, 3)).toEqual({ ...base, x: 15, y: 13 })
	})

	it('resizes from the south-east corner', () => {
		expect(resizeRect(base, 'se', 10, 5)).toEqual({
			x: 10,
			y: 10,
			width: 60,
			height: 25,
		})
	})

	it('resizes from the north-west corner keeping the opposite corner', () => {
		expect(resizeRect(base, 'nw', 5, 5)).toEqual({
			x: 15,
			y: 15,
			width: 45,
			height: 15,
		})
	})

	it('never goes under 1 mm nor outside the page', () => {
		expect(resizeRect(base, 'e', -500, 0).width).toBe(1)
		expect(resizeRect(base, 'w', 500, 0)).toMatchObject({ x: 59, width: 1 })
		expect(resizeRect(base, 'se', 500, 500)).toMatchObject({
			width: 200,
			height: 287,
		})
		expect(resizeRect(base, 'nw', -500, -500)).toMatchObject({ x: 0, y: 0 })
	})

	it('clamps a rectangle that is too big', () => {
		expect(clampRect({ x: 5, y: 5, width: 999, height: 0 })).toEqual({
			x: 0,
			y: 5,
			width: 210,
			height: 1,
		})
	})

	it('rounds to a tenth of a millimeter', () => {
		expect(moveRect(base, 0.04, 0.26)).toMatchObject({ x: 10, y: 10.3 })
	})

	it('nudges by 1 mm, 10 with shift, and ignores other keys', () => {
		expect(nudgeRect(base, 'ArrowRight', false).x).toBe(11)
		expect(nudgeRect(base, 'ArrowLeft', true).x).toBe(0)
		expect(nudgeRect(base, 'ArrowDown', true).y).toBe(20)
		expect(nudgeRect(base, 'ArrowUp', false).y).toBe(9)
		expect(nudgeRect(base, 'a', false)).toBe(base)
	})
})

describe('scope', () => {
	it('matches the preview page', () => {
		const cover = rect({ pages: 'cover' })
		const inner = rect({ pages: 'inner' })
		const all = rect()
		expect([cover, inner, all].map((d) => appliesToPage(d, 0))).toEqual([
			true,
			false,
			true,
		])
		expect([cover, inner, all].map((d) => appliesToPage(d, 2))).toEqual([
			false,
			true,
			true,
		])
	})
})

describe('validation', () => {
	it('accepts an element touching the page edges', () => {
		expect(
			validateDecoration(rect({ x: 160, y: 277, width: 50, height: 20 })),
		).toBeUndefined()
	})

	it('refuses an overflow, a zero size and a bad opacity', () => {
		expect(validateDecoration(rect({ x: 200, width: 20 }))).toBeDefined()
		expect(validateDecoration(rect({ y: 290, height: 20 }))).toBeDefined()
		expect(validateDecoration(rect({ width: 0 }))).toBeDefined()
		expect(validateDecoration(rect({ height: -1 }))).toBeDefined()
		expect(validateDecoration(rect({ opacity: 1.1 }))).toBeDefined()
		expect(validateDecoration(rect({ opacity: -0.1 }))).toBeDefined()
		expect(validateDecoration(rect({ opacity: 0 }))).toBeUndefined()
		expect(validateDecoration(rect({ x: Number.NaN }))).toBeDefined()
	})

	it('refuses a bad color and an image without a key', () => {
		expect(
			validateDecoration({ ...rect(), kind: 'rectangle', fill: 'red' }),
		).toMatch(/#rrggbb/)
		expect(
			validateDecoration({
				kind: 'image',
				key: ' ',
				x: 0,
				y: 0,
				width: 10,
				height: 10,
			}),
		).toBeDefined()
		expect(
			validateDecoration({
				kind: 'image',
				key: 'k',
				x: 0,
				y: 0,
				width: 10,
				height: 10,
			}),
		).toBeUndefined()
	})

	it('refuses more than 20 elements and names the faulty one', () => {
		const many = Array.from({ length: 21 }, () => rect())
		expect(validateDecorations(many)).toMatch(/20/)
		expect(validateDecorations([rect(), rect({ width: 0 })])).toMatch(
			/Élément 2/,
		)
		expect(validateDecorations([rect()])).toBeUndefined()
	})
})
