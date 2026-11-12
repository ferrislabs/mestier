import {
	appliesToPage,
	type Decoration,
	PAGE_HEIGHT_MM,
	PAGE_WIDTH_MM,
	type Rect,
	rectOf,
} from '#/pages/doe-style/lib/decorations'
import { rotatedBounds, rotationOf } from '#/pages/doe-style/lib/transform'

export const SNAP_THRESHOLD_MM = 2

export const PAGE_GUIDES = {
	x: [0, PAGE_WIDTH_MM / 2, PAGE_WIDTH_MM],
	y: [0, PAGE_HEIGHT_MM / 2, PAGE_HEIGHT_MM],
} as const

export interface GuideSet {
	x: number[]
	y: number[]
}

export interface GuideLine {
	axis: 'x' | 'y'
	position: number
}

export interface SnapResult {
	dx: number
	dy: number
	lines: GuideLine[]
}

export function collectGuides(
	decorations: Decoration[],
	pageIndex: number,
	movingIndex: number | null,
	extras: Rect[] = [],
): GuideSet {
	const guides: GuideSet = { x: [...PAGE_GUIDES.x], y: [...PAGE_GUIDES.y] }
	decorations.forEach((decoration, index) => {
		if (index === movingIndex || !appliesToPage(decoration, pageIndex)) return
		const box = rotatedBounds(rectOf(decoration), rotationOf(decoration))
		guides.x.push(box.x, box.x + box.width / 2, box.x + box.width)
		guides.y.push(box.y, box.y + box.height / 2, box.y + box.height)
	})
	for (const box of extras) {
		guides.x.push(box.x, box.x + box.width / 2, box.x + box.width)
		guides.y.push(box.y, box.y + box.height / 2, box.y + box.height)
	}
	return guides
}

function closest(
	candidates: number[],
	guides: number[],
	threshold: number,
): { shift: number; position: number } | null {
	let best: { shift: number; position: number } | null = null
	for (const candidate of candidates) {
		for (const guide of guides) {
			const shift = guide - candidate
			if (Math.abs(shift) > threshold) continue
			if (best === null || Math.abs(shift) < Math.abs(best.shift)) {
				best = { shift, position: guide }
			}
		}
	}
	return best
}

export function snapRect(
	box: Rect,
	guides: GuideSet,
	threshold: number = SNAP_THRESHOLD_MM,
): SnapResult {
	const horizontal = closest(
		[box.x, box.x + box.width / 2, box.x + box.width],
		guides.x,
		threshold,
	)
	const vertical = closest(
		[box.y, box.y + box.height / 2, box.y + box.height],
		guides.y,
		threshold,
	)
	const lines: GuideLine[] = []
	if (horizontal) lines.push({ axis: 'x', position: horizontal.position })
	if (vertical) lines.push({ axis: 'y', position: vertical.position })
	return { dx: horizontal?.shift ?? 0, dy: vertical?.shift ?? 0, lines }
}
