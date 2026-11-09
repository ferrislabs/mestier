import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { UploadItem } from '#/pages/photo-library/lib/uploads'
import { UploadList } from '#/pages/photo-library/ui/upload-list'

const items: UploadItem[] = [
	{ id: '1', name: 'a.jpg', status: 'done', message: null },
	{ id: '2', name: 'b.jpg', status: 'duplicate', message: null },
	{
		id: '3',
		name: 'plan.pdf',
		status: 'rejected',
		message: 'Seules les images',
	},
	{ id: '4', name: 'c.heic', status: 'converting', message: null },
]

describe('UploadList', () => {
	it('shows one state per file', () => {
		render(<UploadList items={items} isBusy onClear={() => {}} />)
		expect(screen.getByText('Envoyée')).toBeTruthy()
		expect(screen.getByText('Déjà dans la photothèque')).toBeTruthy()
		expect(screen.getByText('Seules les images')).toBeTruthy()
		expect(screen.getByText('Conversion…')).toBeTruthy()
	})

	it('only lets the list be cleared once nothing is running', async () => {
		const onClear = vi.fn()
		const { rerender } = render(
			<UploadList items={items} isBusy onClear={onClear} />,
		)
		const button = screen.getByRole('button', { name: /Effacer la liste/ })
		expect((button as HTMLButtonElement).disabled).toBe(true)
		rerender(
			<UploadList items={items.slice(0, 3)} isBusy={false} onClear={onClear} />,
		)
		await userEvent.click(
			screen.getByRole('button', { name: /Effacer la liste/ }),
		)
		expect(onClear).toHaveBeenCalledOnce()
	})

	it('renders nothing without files', () => {
		const { container } = render(
			<UploadList items={[]} isBusy={false} onClear={() => {}} />,
		)
		expect(container.firstChild).toBeNull()
	})
})
