export function saveStatus({
	dirty,
	saved,
	savedLabel,
}: {
	dirty: boolean
	saved: boolean
	savedLabel: string
}): string {
	if (dirty) return 'Modifications non enregistrées.'
	return saved ? savedLabel : ''
}
