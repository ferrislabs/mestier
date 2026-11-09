export const HEIC_FAILURE_MESSAGE =
	"Ce fichier HEIC n'a pas pu être converti, exportez-le en JPEG"
export const NOT_AN_IMAGE_MESSAGE =
	'Seules les images JPEG, PNG, WebP et HEIC sont acceptées.'

const SERVER_IMAGE_TYPES = new Set(['image/jpeg', 'image/png', 'image/webp'])
const HEIC_TYPES = new Set([
	'image/heic',
	'image/heif',
	'image/heic-sequence',
	'image/heif-sequence',
])
const HEIC_EXTENSION = /\.(heic|heif)$/i
const SERVER_IMAGE_EXTENSION = /\.(jpe?g|png|webp)$/i

export type FileKind = 'image' | 'heic' | 'unsupported'

interface FileLike {
	name: string
	type: string
}

export function classifyFile(file: FileLike): FileKind {
	const type = file.type.toLowerCase()
	if (HEIC_TYPES.has(type) || HEIC_EXTENSION.test(file.name)) return 'heic'
	if (SERVER_IMAGE_TYPES.has(type)) return 'image'
	if (type === '' && SERVER_IMAGE_EXTENSION.test(file.name)) return 'image'
	return 'unsupported'
}

export function jpegName(name: string): string {
	return `${name.replace(/\.[^.]+$/, '')}.jpg`
}

export type PrepareOutcome =
	| { kind: 'ready'; file: Blob; name: string }
	| { kind: 'rejected'; message: string }

export async function prepareFile(
	file: File,
	convertHeic: (file: File) => Promise<Blob>,
): Promise<PrepareOutcome> {
	const kind = classifyFile(file)
	if (kind === 'unsupported') {
		return { kind: 'rejected', message: NOT_AN_IMAGE_MESSAGE }
	}
	if (kind === 'image') {
		const type = file.type || 'image/jpeg'
		return {
			kind: 'ready',
			file: file.slice(0, file.size, type),
			name: file.name,
		}
	}
	try {
		const converted = await convertHeic(file)
		return { kind: 'ready', file: converted, name: jpegName(file.name) }
	} catch {
		return { kind: 'rejected', message: HEIC_FAILURE_MESSAGE }
	}
}

export async function convertHeicToJpeg(file: File): Promise<Blob> {
	const { heicTo } = await import('heic-to')
	return heicTo({ blob: file, type: 'image/jpeg', quality: 0.9 })
}
