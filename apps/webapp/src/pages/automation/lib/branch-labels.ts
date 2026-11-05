const BRANCH_LABELS: Record<string, string> = {
	Then: 'vrai',
	Else: 'faux',
	Each: 'chaque élément',
	After: 'terminé',
}

export function branchLabel(branch: string): string {
	return BRANCH_LABELS[branch] ?? branch
}
