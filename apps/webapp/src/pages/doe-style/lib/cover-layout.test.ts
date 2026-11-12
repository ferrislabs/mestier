import { describe, expect, it } from 'vitest'
import {
	COVER_BLOCK_KEYS,
	COVER_BLOCK_LABELS,
	COVER_TITLE_MIN_HEIGHT_MM,
	coverBlockErrors,
	defaultCoverLayout,
	moveBlock,
	resizeBlock,
	setBlockRect,
	setBlockVisible,
	validateCoverLayout,
} from '#/pages/doe-style/lib/cover-layout'
import {
	PAGE_HEIGHT_MM,
	PAGE_WIDTH_MM,
} from '#/pages/doe-style/lib/decorations'

describe('defaultCoverLayout', () => {
	it('lists the six blocks with their French labels', () => {
		expect(COVER_BLOCK_KEYS).toHaveLength(6)
		expect(COVER_BLOCK_LABELS.title).toBe('Bandeau titre')
		expect(COVER_BLOCK_LABELS.meta).toBe('Bloc rédacteur / date / indice')
		expect(Object.keys(defaultCoverLayout()).sort()).toEqual(
			[...COVER_BLOCK_KEYS].sort(),
		)
	})

	it('is valid, visible and inside the page', () => {
		const layout = defaultCoverLayout()
		expect(validateCoverLayout(layout)).toBeUndefined()
		for (const key of COVER_BLOCK_KEYS) {
			const frame = layout[key]
			expect(frame.visible).toBe(true)
			expect(frame.x).toBeGreaterThanOrEqual(20)
			expect(frame.x + frame.width).toBeLessThanOrEqual(PAGE_WIDTH_MM - 20)
			expect(frame.y + frame.height).toBeLessThanOrEqual(PAGE_HEIGHT_MM)
		}
	})

	it('keeps the title tall enough and the photo above the bottom margin', () => {
		const layout = defaultCoverLayout()
		expect(layout.title.height).toBeGreaterThanOrEqual(
			COVER_TITLE_MIN_HEIGHT_MM,
		)
		expect(layout.photo.y + layout.photo.height).toBe(PAGE_HEIGHT_MM - 20)
		expect(layout.logo).toMatchObject({ x: 20, y: 24, width: 50, height: 25 })
		expect(layout.company.x).toBe(130)
		expect(layout.company.width).toBe(60)
	})

	it('stacks the flow without overlap', () => {
		const layout = defaultCoverLayout()
		expect(layout.title.y).toBeGreaterThanOrEqual(
			layout.place.y + layout.place.height,
		)
		expect(layout.meta.y).toBeGreaterThanOrEqual(
			layout.title.y + layout.title.height,
		)
		expect(layout.photo.y).toBeGreaterThanOrEqual(
			layout.meta.y + layout.meta.height,
		)
	})

	it('returns a fresh object each time', () => {
		expect(defaultCoverLayout()).not.toBe(defaultCoverLayout())
	})
})

describe('validateCoverLayout', () => {
	it('accepts null', () => {
		expect(validateCoverLayout(null)).toBeUndefined()
	})

	it('refuses a block outside the page', () => {
		const layout = setBlockRect(defaultCoverLayout(), 'photo', {
			x: 100,
			y: 10,
			width: 120,
			height: 50,
		})
		expect(validateCoverLayout(layout)).toMatch(/^Photo principale : /)
		expect(coverBlockErrors(layout).photo).toMatch(/dépasse/)
		expect(coverBlockErrors(layout).logo).toBeUndefined()
	})

	it('refuses a zero or negative size', () => {
		const base = defaultCoverLayout()
		expect(
			validateCoverLayout({ ...base, place: { ...base.place, width: 0 } }),
		).toMatch(/supérieures à 0/)
		expect(
			validateCoverLayout({ ...base, place: { ...base.place, height: -1 } }),
		).toMatch(/supérieures à 0/)
	})

	it('refuses a negative position and a non finite number', () => {
		const base = defaultCoverLayout()
		expect(
			validateCoverLayout({ ...base, logo: { ...base.logo, x: -1 } }),
		).toMatch(/dans la page/)
		expect(
			validateCoverLayout({ ...base, logo: { ...base.logo, y: Number.NaN } }),
		).toMatch(/nombres/)
	})

	it('accepts a block touching the page edges', () => {
		const layout = setBlockRect(defaultCoverLayout(), 'photo', {
			x: 0,
			y: 0,
			width: PAGE_WIDTH_MM,
			height: PAGE_HEIGHT_MM,
		})
		expect(validateCoverLayout(layout)).toBeUndefined()
	})
})

describe('block helpers', () => {
	it('moves one block without touching the input or the others', () => {
		const base = defaultCoverLayout()
		const snapshot = structuredClone(base)
		const next = moveBlock(base, 'title', 10, 20)
		expect(base).toEqual(snapshot)
		expect(next.title.x).toBe(base.title.x + 10)
		expect(next.title.y).toBe(base.title.y + 20)
		expect(next.title.visible).toBe(true)
		expect(next.logo).toBe(base.logo)
	})

	it('keeps a moved block inside the page', () => {
		const next = moveBlock(defaultCoverLayout(), 'logo', 500, 500)
		expect(validateCoverLayout(next)).toBeUndefined()
		expect(next.logo.x + next.logo.width).toBe(PAGE_WIDTH_MM)
	})

	it('resizes with a handle', () => {
		const base = defaultCoverLayout()
		const next = resizeBlock(base, 'logo', 'se', 10, 5)
		expect(next.logo.x).toBe(base.logo.x)
		expect(next.logo.width).toBe(base.logo.width + 10)
		expect(next.logo.height).toBe(base.logo.height + 5)
		expect(base.logo.width).toBe(50)
	})

	it('toggles visibility and returns the same layout when unchanged', () => {
		const base = defaultCoverLayout()
		expect(setBlockVisible(base, 'place', true)).toBe(base)
		const hidden = setBlockVisible(base, 'place', false)
		expect(hidden.place.visible).toBe(false)
		expect(base.place.visible).toBe(true)
	})
})
