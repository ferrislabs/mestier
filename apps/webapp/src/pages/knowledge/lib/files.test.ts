import { describe, expect, it } from 'vitest'
import {
	fileTypeLabel,
	formatSize,
	MAX_FILE_BYTES,
	mimeTypeOf,
	refusalOf,
	titleFromFileName,
} from '#/pages/knowledge/lib/files'

const file = (name: string, type: string, size = 10) => ({ name, type, size })

describe('knowledge files', () => {
	it('trusts an accepted declared type and falls back on the extension', () => {
		expect(mimeTypeOf(file('a.bin', 'application/pdf'))).toBe('application/pdf')
		expect(mimeTypeOf(file('Devis.PDF', ''))).toBe('application/pdf')
		expect(mimeTypeOf(file('x.docx', 'application/octet-stream'))).toContain(
			'wordprocessingml',
		)
		expect(mimeTypeOf(file('archive.zip', 'application/zip'))).toBeNull()
	})

	it('refuses an empty, a too large or an unknown file before any request', () => {
		expect(refusalOf(file('a.pdf', 'application/pdf', 0))).toMatch(/vide/)
		expect(
			refusalOf(file('a.pdf', 'application/pdf', MAX_FILE_BYTES + 1)),
		).toMatch(/volumineux/)
		expect(refusalOf(file('a.zip', 'application/zip'))).toMatch(/Formats/)
		expect(refusalOf(file('a.pdf', 'application/pdf'))).toBeNull()
	})

	it('makes a title from the file name', () => {
		expect(titleFromFileName('Tarifs_gazon_2026.pdf')).toBe('Tarifs gazon 2026')
		expect(titleFromFileName('.pdf')).toBe('.pdf')
	})

	it('formats sizes and types', () => {
		expect(formatSize(512)).toBe('512 o')
		expect(formatSize(2048)).toBe('2 Ko')
		expect(formatSize(3 * 1024 * 1024)).toBe('3,0 Mo')
		expect(fileTypeLabel('application/pdf')).toBe('PDF')
		expect(fileTypeLabel('image/png')).toBe('Image')
		expect(fileTypeLabel('text/plain')).toBe('Texte')
	})
})
