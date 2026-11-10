import type { Decoration, Rect } from '#/pages/doe-style/lib/decorations'

export const ANGLE_STEP = 15

export interface Point {
	x: number
	y: number
}

export function normalizeRotation(degrees: number): number {
	if (!Number.isFinite(degrees)) return 0
	return ((Math.round(degrees) % 360) + 360) % 360
}

export function rotationOf(decoration: Decoration): number {
	return normalizeRotation(decoration.rotation ?? 0)
}

export function cssTransform(decoration: Decoration): string {
	const sx = decoration.flip_x ? -1 : 1
	const sy = decoration.flip_y ? -1 : 1
	return `rotate(${rotationOf(decoration)}deg) scale(${sx}, ${sy})`
}

export function rotatedBounds(rect: Rect, rotation: number): Rect {
	const radians = (rotation * Math.PI) / 180
	const cos = Math.abs(Math.cos(radians))
	const sin = Math.abs(Math.sin(radians))
	const width = rect.width * cos + rect.height * sin
	const height = rect.width * sin + rect.height * cos
	const centerX = rect.x + rect.width / 2
	const centerY = rect.y + rect.height / 2
	return {
		x: centerX - width / 2,
		y: centerY - height / 2,
		width,
		height,
	}
}

export function angleFromPointer(center: Point, pointer: Point): number {
	const dx = pointer.x - center.x
	const dy = pointer.y - center.y
	const degrees = (Math.atan2(dx, -dy) * 180) / Math.PI
	return degrees < 0 ? degrees + 360 : degrees
}

export function snapAngle(degrees: number, free: boolean): number {
	if (free) return normalizeRotation(degrees)
	return normalizeRotation(Math.round(degrees / ANGLE_STEP) * ANGLE_STEP)
}

export function rotateBy(decoration: Decoration, delta: number): Decoration {
	return {
		...decoration,
		rotation: normalizeRotation(rotationOf(decoration) + delta),
	}
}

export function toggleFlip(
	decoration: Decoration,
	axis: 'x' | 'y',
): Decoration {
	return axis === 'x'
		? { ...decoration, flip_x: !decoration.flip_x }
		: { ...decoration, flip_y: !decoration.flip_y }
}

export function toLocalDelta(
	dx: number,
	dy: number,
	rotation: number,
): { dx: number; dy: number } {
	const radians = (rotation * Math.PI) / 180
	const cos = Math.cos(radians)
	const sin = Math.sin(radians)
	return { dx: dx * cos + dy * sin, dy: -dx * sin + dy * cos }
}
