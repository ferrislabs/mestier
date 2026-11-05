import { type KeyboardEvent, useRef, useState } from 'react'

export type CellNav = {
	cell: string
	tabIndex: 0 | -1
	onFocus: () => void
	onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => void
}

function clamp(value: number, max: number): number {
	return Math.min(Math.max(value, 0), Math.max(max, 0))
}

export function useRovingCells(rows: number, cols: number, initialCol: number) {
	const tableRef = useRef<HTMLTableElement>(null)
	const [active, setActive] = useState({ row: 0, col: initialCol })
	const activeRow = clamp(active.row, rows - 1)
	const activeCol = clamp(active.col, cols - 1)

	function focusCell(row: number, col: number) {
		const nextRow = clamp(row, rows - 1)
		const nextCol = clamp(col, cols - 1)
		setActive({ row: nextRow, col: nextCol })
		tableRef.current
			?.querySelector<HTMLElement>(`[data-cell="${nextRow}:${nextCol}"]`)
			?.focus()
	}

	function navFor(row: number, col: number): CellNav {
		return {
			cell: `${row}:${col}`,
			tabIndex: row === activeRow && col === activeCol ? 0 : -1,
			onFocus: () => setActive({ row, col }),
			onKeyDown: (event) => {
				if (event.altKey || event.ctrlKey || event.metaKey) return
				const moves: Record<string, readonly [number, number]> = {
					ArrowLeft: [row, col - 1],
					ArrowRight: [row, col + 1],
					ArrowUp: [row - 1, col],
					ArrowDown: [row + 1, col],
					Home: [row, 0],
					End: [row, cols - 1],
				}
				const target = Object.hasOwn(moves, event.key)
					? moves[event.key]
					: undefined
				if (!target) return
				event.preventDefault()
				focusCell(target[0], target[1])
			},
		}
	}

	return { tableRef, navFor }
}
