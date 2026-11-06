function isPlainObject(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function flattenInto(
	target: Record<string, string>,
	prefix: string,
	value: unknown,
): void {
	if (isPlainObject(value)) {
		for (const [key, child] of Object.entries(value)) {
			flattenInto(target, prefix ? `${prefix}.${key}` : key, child)
		}
		return
	}
	target[prefix] = typeof value === 'string' ? value : JSON.stringify(value)
}

export function flattenItem(item: unknown): Record<string, string> {
	const row: Record<string, string> = {}
	flattenInto(row, '', item)
	return row
}

export function collectColumns(
	rows: readonly Record<string, string>[],
): string[] {
	const columns = new Set<string>()
	for (const row of rows) {
		for (const key of Object.keys(row)) columns.add(key)
	}
	return [...columns]
}

export function toItems(value: unknown): unknown[] {
	if (value === undefined || value === null) return []
	return Array.isArray(value) ? value : [value]
}

const SCALAR_KEY = 'valeur'

const BRANCH_ORDER = ['Then', 'Else', 'Each', 'After'] as const

export function asItem(value: unknown): unknown {
	return isPlainObject(value) ? value : { [SCALAR_KEY]: value }
}

export interface BranchList {
	branch: string
	items: unknown[]
}

export type StepOutput =
	| { kind: 'main'; items: unknown[] }
	| { kind: 'branched'; lists: BranchList[] }

function isBranched(value: Record<string, unknown>): boolean {
	const keys = Object.keys(value)
	return (
		keys.length > 0 &&
		keys.every(
			(key) =>
				(BRANCH_ORDER as readonly string[]).includes(key) &&
				Array.isArray(value[key]),
		)
	)
}

export function parseStepOutput(output: unknown): StepOutput {
	if (Array.isArray(output)) return { kind: 'main', items: output.map(asItem) }
	if (isPlainObject(output) && isBranched(output)) {
		return {
			kind: 'branched',
			lists: BRANCH_ORDER.flatMap((branch) => {
				const list = output[branch]
				return Array.isArray(list) ? [{ branch, items: list.map(asItem) }] : []
			}),
		}
	}
	if (output === undefined || output === null)
		return { kind: 'main', items: [] }
	return { kind: 'main', items: [asItem(output)] }
}

export function outputItems(
	output: unknown,
	feeding: readonly string[],
): unknown[] {
	const parsed = parseStepOutput(output)
	if (parsed.kind === 'main') return parsed.items
	return parsed.lists
		.filter((list) => feeding.length === 0 || feeding.includes(list.branch))
		.flatMap((list) => list.items)
}

export function outputItemCount(output: unknown): number {
	return outputItems(output, []).length
}

export function triggerItem(payload: unknown): unknown {
	return asItem(payload ?? {})
}
