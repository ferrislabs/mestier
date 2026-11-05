const timeFormatter = new Intl.DateTimeFormat('fr-FR', {
	hour: '2-digit',
	minute: '2-digit',
})

export function formatTime(date: Date): string {
	return timeFormatter.format(date)
}

export function getInitials(name: string): string {
	return name
		.split(' ')
		.filter((part) => part !== '')
		.slice(0, 2)
		.map((part) => part.charAt(0).toUpperCase())
		.join('')
}

export function userHue(userId: string | null | undefined): number {
	if (!userId) return 0
	let hash = 0
	for (const char of userId) hash = (hash * 31 + char.charCodeAt(0)) % 360
	return hash
}

function roundedOneDecimal(value: number): string {
	return String(Math.round(value * 10) / 10).replace('.', ',')
}

export function formatFileSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} o`
	const kibioctets = bytes / 1024
	if (kibioctets < 1024) return `${roundedOneDecimal(kibioctets)} Ko`
	return `${roundedOneDecimal(kibioctets / 1024)} Mo`
}
