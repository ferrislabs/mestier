const IDENTIFIER = /^[A-Za-z_$][\w$]*$/

function property(key: string): string {
	return IDENTIFIER.test(key) ? key : JSON.stringify(key)
}

function typeOf(value: unknown, indent: string): string {
	if (value === null || value === undefined) return 'null'
	if (Array.isArray(value)) {
		const first = value[0]
		return first === undefined ? 'unknown[]' : `${typeOf(first, indent)}[]`
	}
	switch (typeof value) {
		case 'string':
			return 'string'
		case 'number':
			return 'number'
		case 'boolean':
			return 'boolean'
		default:
			return objectType(value as Record<string, unknown>, indent)
	}
}

function objectType(value: Record<string, unknown>, indent: string): string {
	const entries = Object.entries(value)
	if (entries.length === 0) return '{}'
	const inner = `${indent}  `
	const lines = entries.map(
		([key, child]) => `${inner}${property(key)}: ${typeOf(child, inner)}`,
	)
	return `{\n${lines.join('\n')}\n${indent}}`
}

export function payloadTypeName(eventName: string): string {
	const pascal = eventName
		.split(/[^A-Za-z0-9]+/)
		.filter((part) => part !== '')
		.map((part) => part[0].toUpperCase() + part.slice(1))
		.join('')
	return `${pascal}Payload`
}

export function payloadContractSource(
	eventName: string,
	example: unknown,
): string {
	return `export interface ${payloadTypeName(eventName)} ${objectType(
		typeof example === 'object' && example !== null && !Array.isArray(example)
			? (example as Record<string, unknown>)
			: {},
		'',
	)}`
}
