import { describe, expect, it } from 'vitest'
import type { Decoration } from '#/pages/doe-style/lib/decorations'
import {
	collectGuides,
	PAGE_GUIDES,
	SNAP_THRESHOLD_MM,
	snapRect,
} from '#/pages/doe-style/lib/snap'

type Transformed = Pick<
	Decoration,
	'x' | 'y' | 'width' | 'height' | 'rotation' | 'flip_x' | 'flip_y' | 'pages'
>

const rect = (over: Partial<Transformed> = {}): Decoration => ({
	kind: 'rectangle',
	fill: '#112233',
	x: 10,
	y: 10,
	width: 40,
	height: 10,
	...over,
})

describe('collectGuides', () => {
	it('always contains the page edges and center', () => {
		const guides = collectGuides([], 0, null)
		expect(guides.x).toEqual([...PAGE_GUIDES.x])
		expect(guides.y).toEqual([...PAGE_GUIDES.y])
		expect(PAGE_GUIDES.x).toEqual([0, 105, 210])
		expect(PAGE_GUIDES.y).toEqual([0, 148.5, 297])
	})

	it('adds edges and centers of the other elements of the page', () => {
		const guides = collectGuides([rect()], 0, null)
		expect(guides.x).toEqual(expect.arrayContaining([10, 30, 50]))
		expect(guides.y).toEqual(expect.arrayContaining([10, 15, 20]))
	})

	it('uses the rotated bounding box', () => {
		const guides = collectGuides([rect({ rotation: 90 })], 0, null)
		expect(guides.x.some((value) => Math.abs(value - 25) < 1e-6)).toBe(true)
		expect(guides.y.some((value) => Math.abs(value - 35) < 1e-6)).toBe(true)
	})

	it('skips the moving element and elements of other pages', () => {
		const list = [rect(), rect({ x: 100, pages: 'inner' })]
		expect(collectGuides(list, 0, 0).x).toEqual([...PAGE_GUIDES.x])
		expect(collectGuides(list, 0, null).x).not.toContain(100)
		expect(collectGuides(list, 1, 0).x).toContain(100)
	})
})

describe('snapRect', () => {
	const guides = collectGuides([], 0, null)

	it('sticks the center to the page center within the threshold', () => {
		expect(SNAP_THRESHOLD_MM).toBe(2)
		const snapped = snapRect({ x: 84.5, y: 100, width: 40, height: 10 }, guides)
		expect(snapped.dx).toBeCloseTo(0.5)
		expect(snapped.dy).toBe(0)
		expect(snapped.lines).toEqual([{ axis: 'x', position: 105 }])
	})

	it('does nothing beyond the threshold', () => {
		const snapped = snapRect({ x: 80, y: 100, width: 40, height: 10 }, guides)
		expect(snapped).toEqual({ dx: 0, dy: 0, lines: [] })
	})

	it('snaps both axes and picks the closest candidate', () => {
		const snapped = snapRect({ x: 1, y: 0.5, width: 40, height: 10 }, guides)
		expect(snapped.dx).toBe(-1)
		expect(snapped.dy).toBe(-0.5)
		expect(snapped.lines).toHaveLength(2)
	})

	it('snaps the right edge to the page edge', () => {
		const snapped = snapRect({ x: 169, y: 100, width: 40, height: 10 }, guides)
		expect(snapped.dx).toBe(1)
		expect(snapped.lines).toEqual([{ axis: 'x', position: 210 }])
	})
})
