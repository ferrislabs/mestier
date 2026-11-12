export function formatAdminDate(iso: string | null | undefined): string {
	if (!iso) return '–'
	return new Intl.DateTimeFormat('fr-FR', { dateStyle: 'medium' }).format(
		new Date(iso),
	)
}

export function formatAdminDateTime(iso: string | null | undefined): string {
	if (!iso) return '–'
	return new Intl.DateTimeFormat('fr-FR', {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(new Date(iso))
}

export function formatAdminNumber(value: number): string {
	return new Intl.NumberFormat('fr-FR').format(value)
}
