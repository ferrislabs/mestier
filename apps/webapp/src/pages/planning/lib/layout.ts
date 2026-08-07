export interface MinuteSpan {
	startMinute: number
	endMinute: number
}

export interface SegmentPosition {
	left: number
	width: number
}

/**
 * `left`/`width` as percentages of `axis`, clamping a span that starts
 * before or ends after the axis instead of overflowing — the case of an
 * entry straddling the visible window's bounds. An all-day entry, given the
 * full `[00:00, 24:00)` span, clamps down to exactly `{ left: 0, width: 100 }`
 * through the same code path — no separate case needed.
 */
export function computeSegmentPosition(
	span: MinuteSpan,
	axis: MinuteSpan,
): SegmentPosition {
	const axisSpan = axis.endMinute - axis.startMinute
	if (axisSpan <= 0) return { left: 0, width: 0 }

	const clampedStart = Math.max(span.startMinute, axis.startMinute)
	const clampedEnd = Math.min(span.endMinute, axis.endMinute)
	if (clampedEnd <= clampedStart) return { left: 0, width: 0 }

	return {
		left: ((clampedStart - axis.startMinute) / axisSpan) * 100,
		width: ((clampedEnd - clampedStart) / axisSpan) * 100,
	}
}

export interface StackedItem<T> {
	item: T
	row: number
}

/**
 * Assigns each item a row so that overlapping spans never share one — no
 * geometric collision resolution, just an extra row per overlap (see the
 * planning design doc's "une grille, trois granularités" section). A greedy
 * sweep by start time, reusing the first row whose last item has already
 * ended.
 */
export function stackOverlapping<T>(
	items: T[],
	getSpan: (item: T) => MinuteSpan,
): StackedItem<T>[] {
	const sorted = [...items].sort(
		(a, b) => getSpan(a).startMinute - getSpan(b).startMinute,
	)
	const rowEnds: number[] = []
	const result: StackedItem<T>[] = []

	for (const item of sorted) {
		const span = getSpan(item)
		let row = rowEnds.findIndex((end) => end <= span.startMinute)
		if (row === -1) {
			row = rowEnds.length
			rowEnds.push(span.endMinute)
		} else {
			rowEnds[row] = span.endMinute
		}
		result.push({ item, row })
	}

	return result
}
