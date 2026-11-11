export interface CellPosition {
	row: number
	column: number
}

export interface GridBounds {
	rows: number
	columns: number
}

export type NavigationKey =
	| 'ArrowUp'
	| 'ArrowDown'
	| 'ArrowLeft'
	| 'ArrowRight'
	| 'Tab'
	| 'ShiftTab'

function clamp(value: number, max: number): number {
	return Math.max(0, Math.min(value, max))
}

export function moveActiveCell(
	active: CellPosition,
	key: string,
	bounds: GridBounds,
): CellPosition | null {
	const lastRow = Math.max(bounds.rows - 1, 0)
	const lastColumn = Math.max(bounds.columns - 1, 0)
	switch (key) {
		case 'ArrowUp':
			return { row: clamp(active.row - 1, lastRow), column: active.column }
		case 'ArrowDown':
			return { row: clamp(active.row + 1, lastRow), column: active.column }
		case 'ArrowLeft':
			return { row: active.row, column: clamp(active.column - 1, lastColumn) }
		case 'ArrowRight':
			return { row: active.row, column: clamp(active.column + 1, lastColumn) }
		case 'Tab':
			if (active.column < lastColumn) {
				return { row: active.row, column: active.column + 1 }
			}
			return active.row < lastRow ? { row: active.row + 1, column: 0 } : active
		case 'ShiftTab':
			if (active.column > 0) {
				return { row: active.row, column: active.column - 1 }
			}
			return active.row > 0
				? { row: active.row - 1, column: lastColumn }
				: active
		default:
			return null
	}
}
