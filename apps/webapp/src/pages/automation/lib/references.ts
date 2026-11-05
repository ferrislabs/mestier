export function uniqueLabel(label: string, existing: Iterable<string>): string {
	const taken = new Set(existing)
	if (!taken.has(label)) return label
	for (let suffix = 2; ; suffix += 1) {
		const candidate = `${label} ${suffix}`
		if (!taken.has(candidate)) return candidate
	}
}

export function labelConflict(
	label: string,
	nodeId: string,
	nodes: readonly { id: string; data: { label: string } }[],
): string | null {
	const trimmed = label.trim()
	if (trimmed === '') return 'Le nom ne peut pas être vide.'
	return nodes.some((node) => node.id !== nodeId && node.data.label === trimmed)
		? 'Un autre nœud porte déjà ce nom.'
		: null
}
