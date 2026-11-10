import { describe, expect, it } from 'vitest'
import type { Decoration } from '#/pages/doe-style/lib/decorations'
import {
	ANGLE_STEP,
	angleFromPointer,
	cssTransform,
	normalizeRotation,
	rotateBy,
	rotatedBounds,
	rotationOf,
	snapAngle,
	toggleFlip,
	toLocalDelta,
} from '#/pages/doe-style/lib/transform'

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

describe('normalizeRotation', () => {
	it('rounds and wraps like the backend', () => {
		expect(normalizeRotation(-90)).toBe(270)
		expect(normalizeRotation(359.6)).toBe(0)
		expect(normalizeRotation(360)).toBe(0)
		expect(normalizeRotation(12.4)).toBe(12)
		expect(normalizeRotation(725)).toBe(5)
	})

	it('maps non finite values to 0', () => {
		expect(normalizeRotation(Number.NaN)).toBe(0)
		expect(normalizeRotation(Number.POSITIVE_INFINITY)).toBe(0)
	})
})

describe('cssTransform', () => {
	it('is the identity for a plain decoration', () => {
		expect(rotationOf(rect())).toBe(0)
		expect(cssTransform(rect())).toBe('rotate(0deg) scale(1, 1)')
	})

	it('rotates 90 degrees with a horizontal flip', () => {
		expect(cssTransform(rect({ rotation: 90, flip_x: true }))).toBe(
			'rotate(90deg) scale(-1, 1)',
		)
	})

	it('flips vertically', () => {
		expect(cssTransform(rect({ flip_y: true }))).toBe(
			'rotate(0deg) scale(1, -1)',
		)
	})
})

describe('rotatedBounds', () => {
	it('swaps width and height at 90 degrees around the same center', () => {
		const bounds = rotatedBounds({ x: 10, y: 10, width: 40, height: 10 }, 90)
		expect(bounds.width).toBeCloseTo(10)
		expect(bounds.height).toBeCloseTo(40)
		expect(bounds.x + bounds.width / 2).toBeCloseTo(30)
		expect(bounds.y + bounds.height / 2).toBeCloseTo(15)
	})

	it('is unchanged at 0 and 180 degrees', () => {
		const frame = { x: 10, y: 10, width: 40, height: 10 }
		expect(rotatedBounds(frame, 0)).toEqual(frame)
		const turned = rotatedBounds(frame, 180)
		expect(turned.width).toBeCloseTo(40)
		expect(turned.height).toBeCloseTo(10)
	})

	it('grows to the diagonal box at 45 degrees', () => {
		const bounds = rotatedBounds({ x: 0, y: 0, width: 10, height: 10 }, 45)
		expect(bounds.width).toBeCloseTo(Math.SQRT2 * 10)
	})
})

describe('angleFromPointer', () => {
	const center = { x: 100, y: 100 }

	it('is 0 straight above the center', () => {
		expect(angleFromPointer(center, { x: 100, y: 50 })).toBe(0)
	})

	it('is clockwise positive', () => {
		expect(angleFromPointer(center, { x: 150, y: 100 })).toBe(90)
		expect(angleFromPointer(center, { x: 100, y: 150 })).toBe(180)
		expect(angleFromPointer(center, { x: 50, y: 100 })).toBe(270)
	})
})

describe('snapAngle', () => {
	it('snaps to 15 degree steps', () => {
		expect(ANGLE_STEP).toBe(15)
		expect(snapAngle(52, false)).toBe(45)
		expect(snapAngle(53, false)).toBe(60)
		expect(snapAngle(358, false)).toBe(0)
	})

	it('is free when requested', () => {
		expect(snapAngle(52.4, true)).toBe(52)
	})
})

describe('rotateBy and toggleFlip', () => {
	it('adds a delta and normalizes', () => {
		expect(rotateBy(rect({ rotation: 10 }), -15).rotation).toBe(355)
		expect(rotateBy(rect({ rotation: 350 }), 15).rotation).toBe(5)
	})

	it('toggles each flip independently', () => {
		const once = toggleFlip(rect(), 'x')
		expect(once.flip_x).toBe(true)
		expect(once.flip_y).toBeUndefined()
		expect(toggleFlip(once, 'x').flip_x).toBe(false)
		expect(toggleFlip(rect({ flip_y: true }), 'y').flip_y).toBe(false)
	})
})

describe('toLocalDelta', () => {
	it('is the identity without rotation', () => {
		expect(toLocalDelta(3, 4, 0)).toEqual({ dx: 3, dy: 4 })
	})

	it('projects a screen move on the rotated axes', () => {
		const local = toLocalDelta(0, 10, 90)
		expect(local.dx).toBeCloseTo(10)
		expect(local.dy).toBeCloseTo(0)
	})
})
