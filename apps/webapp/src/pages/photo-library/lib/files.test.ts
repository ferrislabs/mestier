import { describe, expect, it, vi } from 'vitest'
import {
	classifyFile,
	HEIC_FAILURE_MESSAGE,
	jpegName,
	NOT_AN_IMAGE_MESSAGE,
	prepareFile,
} from '#/pages/photo-library/lib/files'

function file(name: string, type: string): File {
	return new File(['x'], name, { type })
}

describe('classifyFile', () => {
	it('recognises HEIC by extension or by MIME type', () => {
		expect(classifyFile(file('IMG_1.HEIC', ''))).toBe('heic')
		expect(classifyFile(file('IMG_2.heif', 'application/octet-stream'))).toBe(
			'heic',
		)
		expect(classifyFile(file('photo', 'image/heic'))).toBe('heic')
	})

	it('accepts the three formats the server decodes', () => {
		expect(classifyFile(file('a.jpg', 'image/jpeg'))).toBe('image')
		expect(classifyFile(file('a.png', 'image/png'))).toBe('image')
		expect(classifyFile(file('a.webp', 'image/webp'))).toBe('image')
		expect(classifyFile(file('a.jpeg', ''))).toBe('image')
	})

	it('refuses everything else', () => {
		expect(classifyFile(file('plan.pdf', 'application/pdf'))).toBe(
			'unsupported',
		)
		expect(classifyFile(file('a.gif', 'image/gif'))).toBe('unsupported')
		expect(classifyFile(file('notes.txt', 'text/plain'))).toBe('unsupported')
	})
})

describe('jpegName', () => {
	it('swaps the extension', () => {
		expect(jpegName('IMG_1.HEIC')).toBe('IMG_1.jpg')
		expect(jpegName('sans-extension')).toBe('sans-extension.jpg')
	})
})

describe('prepareFile', () => {
	it('never calls the converter for a plain image', async () => {
		const convert = vi.fn()
		const outcome = await prepareFile(file('a.jpg', 'image/jpeg'), convert)
		expect(outcome.kind).toBe('ready')
		expect(convert).not.toHaveBeenCalled()
	})

	it('converts a HEIC file to a JPEG named .jpg', async () => {
		const jpeg = new Blob(['jpeg'], { type: 'image/jpeg' })
		const convert = vi.fn().mockResolvedValue(jpeg)
		const outcome = await prepareFile(file('IMG_1.heic', 'image/heic'), convert)
		expect(convert).toHaveBeenCalledOnce()
		expect(outcome).toEqual({ kind: 'ready', file: jpeg, name: 'IMG_1.jpg' })
	})

	it('rejects a HEIC file whose conversion fails', async () => {
		const convert = vi.fn().mockRejectedValue(new Error('boom'))
		const outcome = await prepareFile(file('IMG_1.heic', 'image/heic'), convert)
		expect(outcome).toEqual({ kind: 'rejected', message: HEIC_FAILURE_MESSAGE })
		expect(HEIC_FAILURE_MESSAGE).toBe(
			"Ce fichier HEIC n'a pas pu être converti, exportez-le en JPEG",
		)
	})

	it('rejects a PDF before any upload', async () => {
		const convert = vi.fn()
		const outcome = await prepareFile(
			file('plan.pdf', 'application/pdf'),
			convert,
		)
		expect(outcome).toEqual({ kind: 'rejected', message: NOT_AN_IMAGE_MESSAGE })
		expect(convert).not.toHaveBeenCalled()
	})

	it('settles five dropped files as four ready and one rejected PDF', async () => {
		const convert = vi
			.fn()
			.mockResolvedValue(new Blob(['j'], { type: 'image/jpeg' }))
		const dropped = [
			file('a.jpg', 'image/jpeg'),
			file('b.png', 'image/png'),
			file('c.webp', 'image/webp'),
			file('d.heic', 'image/heic'),
			file('e.pdf', 'application/pdf'),
		]
		const outcomes = await Promise.all(
			dropped.map((entry) => prepareFile(entry, convert)),
		)
		expect(outcomes.filter((o) => o.kind === 'ready')).toHaveLength(4)
		expect(outcomes.filter((o) => o.kind === 'rejected')).toHaveLength(1)
	})
})
