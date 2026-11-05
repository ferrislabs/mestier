import type { CollectionSource } from '#/pages/automation/lib/collections'

const DATE_FORMAT = new Intl.DateTimeFormat('fr-FR', {
	day: 'numeric',
	month: 'long',
	year: 'numeric',
})

export function describeSource(source: CollectionSource): string {
	return source.kind === 'url'
		? `Importée depuis ${source.url}`
		: `Importée depuis le fichier ${source.fileName}`
}

export function formatImportDate(isoDate: string): string {
	const date = new Date(isoDate)
	return Number.isNaN(date.getTime()) ? '' : DATE_FORMAT.format(date)
}

export function operationCount(count: number): string {
	return `${count} opération${count > 1 ? 's' : ''}`
}
