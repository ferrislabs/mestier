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
