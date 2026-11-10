export const ACCEPT =
	'application/pdf,image/jpeg,image/png,image/webp,text/plain,text/csv,.docx,.xlsx,.pdf,.txt,.csv'

export const MAX_FILE_BYTES = 25 * 1024 * 1024

const MIME_BY_EXTENSION: Record<string, string> = {
	pdf: 'application/pdf',
	jpg: 'image/jpeg',
	jpeg: 'image/jpeg',
	png: 'image/png',
	webp: 'image/webp',
	txt: 'text/plain',
	csv: 'text/csv',
	docx: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
	xlsx: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
}

const ACCEPTED_TYPES = new Set(Object.values(MIME_BY_EXTENSION))

interface FileLike {
	name: string
	type: string
	size: number
}

/** The type the server will be told, from the browser's or from the extension. */
export function mimeTypeOf(
	file: Pick<FileLike, 'name' | 'type'>,
): string | null {
	const declared = file.type.toLowerCase()
	if (ACCEPTED_TYPES.has(declared)) return declared
	const extension = file.name.split('.').pop()?.toLowerCase() ?? ''
	return MIME_BY_EXTENSION[extension] ?? null
}

/** Why a file is refused before any request, or `null` when it can go. */
export function refusalOf(file: FileLike): string | null {
	if (file.size === 0) return 'Le fichier est vide.'
	if (file.size > MAX_FILE_BYTES)
		return 'Fichier trop volumineux (25 Mo au plus).'
	if (mimeTypeOf(file) === null) {
		return 'Formats acceptés : PDF, Word, Excel, image, texte.'
	}
	return null
}

export function titleFromFileName(name: string): string {
	const title = name
		.replace(/\.[^.]+$/, '')
		.replace(/[_]+/g, ' ')
		.trim()
	return title === '' ? name : title
}

export function formatSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} o`
	if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} Ko`
	return `${(bytes / (1024 * 1024)).toFixed(1).replace('.', ',')} Mo`
}

export function fileTypeLabel(mimeType: string): string {
	if (mimeType === 'application/pdf') return 'PDF'
	if (mimeType.startsWith('image/')) return 'Image'
	if (mimeType.includes('wordprocessingml')) return 'Word'
	if (mimeType.includes('spreadsheetml')) return 'Excel'
	if (mimeType === 'text/csv') return 'CSV'
	return 'Texte'
}
