export function formatIssuedOn(value: string): string {
	const [year, month, day] = value.slice(0, 10).split('-')
	return `${day}/${month}/${year}`
}

export function formatUpdatedAt(value: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: '2-digit',
		year: 'numeric',
	}).format(new Date(value))
}
