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

export function variableExpression(
	root: string,
	path: readonly PathSegment[],
): string {
	return `{{ ${root}${accessor(path)} }}`
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

const REFERENCE =
	/connectors\.([A-Za-z0-9_-]+)\.output((?:\.[A-Za-z_$][\w$]*|\["(?:[^"\\]|\\.)*"\]|\[\d+\])*)/g

export interface ConnectorReference {
	connectorId: string
	accessor: string
}

export function findConnectorReferences(text: string): ConnectorReference[] {
	return [...text.matchAll(REFERENCE)].map((match) => ({
		connectorId: match[1],
		accessor: match[2],
	}))
}

export function displayPath(path: string): string {
	return path.replace(/^\./, '').replace(/\["((?:[^"\\]|\\.)*)"\]/g, '.$1')
}
