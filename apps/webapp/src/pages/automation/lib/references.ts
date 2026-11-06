import {
	findNodeReferences,
	nodeReference,
} from '#/pages/automation/lib/variables'

function jsonEscaped(text: string): string {
	return JSON.stringify(text).slice(1, -1)
}

function replaceAll(text: string, search: string, replacement: string): string {
	return search === replacement ? text : text.split(search).join(replacement)
}

export function renameReferencesInText(
	text: string,
	oldLabel: string,
	newLabel: string,
): string {
	const before = nodeReference(oldLabel)
	const after = nodeReference(newLabel)
	return replaceAll(
		replaceAll(text, before, after),
		jsonEscaped(before),
		jsonEscaped(after),
	)
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function renameReferencesInValue(
	value: unknown,
	oldLabel: string,
	newLabel: string,
): unknown {
	if (typeof value === 'string') {
		return renameReferencesInText(value, oldLabel, newLabel)
	}
	if (Array.isArray(value)) {
		const renamed = value.map((entry) =>
			renameReferencesInValue(entry, oldLabel, newLabel),
		)
		return renamed.every((entry, index) => entry === value[index])
			? value
			: renamed
	}
	if (isPlainObject(value)) {
		const entries = Object.entries(value)
		const renamed = entries.map(
			([key, entry]) =>
				[key, renameReferencesInValue(entry, oldLabel, newLabel)] as const,
		)
		return renamed.every(([, entry], index) => entry === entries[index][1])
			? value
			: Object.fromEntries(renamed)
	}
	return value
}

export function referencedLabels(value: unknown): Set<string> {
	const labels = new Set<string>()
	const visit = (entry: unknown) => {
		if (typeof entry === 'string') {
			for (const reference of findNodeReferences(entry)) {
				labels.add(reference.label)
			}
		} else if (Array.isArray(entry)) {
			entry.forEach(visit)
		} else if (isPlainObject(entry)) {
			Object.values(entry).forEach(visit)
		}
	}
	visit(value)
	return labels
}

function comparable(label: string): string {
	return label.trim().toLowerCase()
}

export function uniqueLabel(label: string, existing: Iterable<string>): string {
	const taken = new Set(Array.from(existing, comparable))
	if (!taken.has(comparable(label))) return label
	for (let suffix = 2; ; suffix += 1) {
		const candidate = `${label} ${suffix}`
		if (!taken.has(comparable(candidate))) return candidate
	}
}

export function labelConflict(
	label: string,
	nodeId: string,
	nodes: readonly { id: string; data: { label: string } }[],
): string | null {
	const trimmed = label.trim()
	if (trimmed === '') return 'Le nom ne peut pas être vide.'
	const wanted = comparable(trimmed)
	return nodes.some(
		(node) => node.id !== nodeId && comparable(node.data.label) === wanted,
	)
		? 'Un autre nœud porte déjà ce nom.'
		: null
}
