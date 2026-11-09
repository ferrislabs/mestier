import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { LibraryPhoto } from '#/hooks/use-photo-library'
import { DoePhotoLibraryPickerDialog } from '#/pages/doe/ui/doe-photo-library-picker-dialog'

function photo(id: string, caption: string | null): LibraryPhoto {
	return {
		id,
		file_key: `library/o/${id}.jpg`,
		thumbnail_key: `library/o/${id}.thumb.jpg`,
		mime_type: 'image/jpeg',
		width: 100,
		height: 100,
		size_bytes: 10,
		caption,
		tags: [],
		project_id: null,
		taken_at: null,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}
}

function renderDialog(
	overrides: Partial<Parameters<typeof DoePhotoLibraryPickerDialog>[0]> = {},
) {
	const props = {
		open: true,
		photos: [photo('p1', 'Terrasse'), photo('p2', null)],
		thumbnailUrls: {},
		selectedIds: new Set<string>(),
		search: '',
		isLoading: false,
		isPlacing: false,
		hasPrevious: false,
		hasNext: false,
		error: null,
		failures: [],
		onSearchChange: vi.fn(),
		onToggle: vi.fn(),
		onPrevious: vi.fn(),
		onNext: vi.fn(),
		onConfirm: vi.fn(),
		onOpenChange: vi.fn(),
		...overrides,
	}
	render(<DoePhotoLibraryPickerDialog {...props} />)
	return props
}

describe('DoePhotoLibraryPickerDialog', () => {
	it('toggles a photo on click', async () => {
		const props = renderDialog()
		await userEvent.click(screen.getByRole('button', { name: 'Terrasse' }))
		expect(props.onToggle).toHaveBeenCalledWith(props.photos[0])
	})

	it('disables placing until something is selected', () => {
		renderDialog()
		const place = screen.getByRole('button', { name: 'Placer' })
		expect((place as HTMLButtonElement).disabled).toBe(true)
	})

	it('counts the selection and confirms', async () => {
		const props = renderDialog({ selectedIds: new Set(['p1', 'p2']) })
		await userEvent.click(
			screen.getByRole('button', { name: 'Placer 2 photos' }),
		)
		expect(props.onConfirm).toHaveBeenCalledOnce()
	})

	it('lists the failed placements', () => {
		renderDialog({
			failures: [
				{
					photoId: 'p2',
					label: 'Photo sans légende',
					message: 'Plafond atteint',
				},
			],
		})
		expect(screen.getByRole('alert').textContent).toContain('Plafond atteint')
	})
})
