export interface CursorSelection {
	start: number
	end: number
}

export interface InsertionResult {
	text: string
	cursor: number
}

export function expressionToken(path: string): string {
	return `{{ ${path} }}`
}

export function insertExpressionAtCursor(
	value: string,
	path: string,
	selection: CursorSelection,
): InsertionResult {
	const token = expressionToken(path)
	const start = Math.min(Math.max(selection.start, 0), value.length)
	const end = Math.min(Math.max(selection.end, start), value.length)

	const text = value.slice(0, start) + token + value.slice(end)

	return { text, cursor: start + token.length }
}
