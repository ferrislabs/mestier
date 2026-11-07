export type PathSegment = string | number

export type VariableType =
	| 'texte'
	| 'nombre'
	| 'booléen'
	| 'objet'
	| 'liste'
	| 'vide'

export interface Variable {
	path: readonly PathSegment[]
	key: string
	type: VariableType
	preview: string
	depth: number
}

const IDENTIFIER = /^[A-Za-z_$][\w$]*$/
const ACCESSOR = /^(?:\.([A-Za-z_$][\w$]*)|\["((?:[^"\\]|\\.)*)"\]|\[(\d+)\])/

function typeOf(value: unknown): VariableType {
	if (value === null || value === undefined) return 'vide'
	if (Array.isArray(value)) return 'liste'
	switch (typeof value) {
		case 'string':
			return 'texte'
		case 'number':
			return 'nombre'
		case 'boolean':
			return 'booléen'
		default:
			return 'objet'
	}
}

function preview(value: unknown): string {
	if (Array.isArray(value)) {
		return `${value.length} élément${value.length > 1 ? 's' : ''}`
	}
	if (typeof value === 'object' && value !== null) return ''
	return String(value)
}

export function listVariables(item: unknown, maxDepth = 5): Variable[] {
	const variables: Variable[] = []
	const visit = (value: unknown, path: PathSegment[], depth: number) => {
		if (depth > maxDepth) return
		if (Array.isArray(value)) {
			const first = value[0]
			if (first === undefined) return
			const childPath = [...path, 0]
			variables.push({
				path: childPath,
				key: '[0]',
				type: typeOf(first),
				preview: preview(first),
				depth,
			})
			visit(first, childPath, depth + 1)
			return
		}
		if (typeof value !== 'object' || value === null) return
		for (const [key, child] of Object.entries(value)) {
			const childPath = [...path, key]
			variables.push({
				path: childPath,
				key,
				type: typeOf(child),
				preview: preview(child),
				depth,
			})
			visit(child, childPath, depth + 1)
		}
	}
	visit(item, [], 0)
	return variables
}

export function accessor(path: readonly PathSegment[]): string {
	return path
		.map((segment) =>
			typeof segment === 'number'
				? `[${segment}]`
				: IDENTIFIER.test(segment)
					? `.${segment}`
					: `["${segment.replaceAll('"', '\\"')}"]`,
		)
		.join('')
}

export function escapeNodeLabel(label: string): string {
	return label.replaceAll('\\', '\\\\').replaceAll("'", "\\'")
}

function unescapeNodeLabel(escaped: string): string {
	return escaped.replaceAll("\\'", "'").replaceAll('\\\\', '\\')
}

export function nodeReference(label: string): string {
	return `$('${escapeNodeLabel(label)}')`
}

export function variableExpression(
	nodeLabel: string,
	path: readonly PathSegment[],
): string {
	return `{{ ${nodeReference(nodeLabel)}.item.json${accessor(path)} }}`
}

export function readPath(value: unknown, rest: string): unknown {
	let current = value
	let remaining = rest
	while (remaining.length > 0) {
		const match = remaining.match(ACCESSOR)
		if (
			!match ||
			current === undefined ||
			current === null ||
			typeof current !== 'object'
		) {
			return undefined
		}
		const key: string | number =
			match[1] ??
			(match[2] !== undefined
				? match[2].replaceAll('\\"', '"')
				: Number(match[3]))
		current = Array.isArray(current)
			? typeof key === 'number'
				? current[key]
				: undefined
			: (current as Record<string, unknown>)[String(key)]
		remaining = remaining.slice(match[0].length)
	}
	return current
}

const NODE_ROOT = /\$\('((?:[^'\\]|\\.)*)'\)/g
const ITEM_PATH =
	/^\.(?:item|first\(\)|last\(\))(?:\.json(?![\w$]))?((?:\.[A-Za-z_$][\w$]*|\["(?:[^"\\]|\\.)*"\]|\[\d+\])*)/
const EXPRESSION = /\{\{([\s\S]*?)\}\}/g
const LEGACY_ROOT = /\bconnectors\s*\.\s*[A-Za-z0-9_-]+\s*\.\s*output\b/

export interface NodeReference {
	label: string
	accessor: string | null
}

function expressionsOf(text: string): string[] {
	return [...text.matchAll(EXPRESSION)].map((match) => match[1])
}

export function findNodeReferences(text: string): NodeReference[] {
	return expressionsOf(text).flatMap((expression) =>
		[...expression.matchAll(NODE_ROOT)].map((match) => {
			const rest = expression.slice((match.index ?? 0) + match[0].length)
			const path = ITEM_PATH.exec(rest)
			return {
				label: unescapeNodeLabel(match[1]),
				accessor: path ? path[1] : null,
			}
		}),
	)
}

export function usesLegacyRoot(text: string): boolean {
	return expressionsOf(text).some((expression) => LEGACY_ROOT.test(expression))
}

export function displayPath(path: string): string {
	return path.replace(/^\./, '').replace(/\["((?:[^"\\]|\\.)*)"\]/g, '.$1')
}
