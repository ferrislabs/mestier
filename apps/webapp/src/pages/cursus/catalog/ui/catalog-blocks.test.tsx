import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { EstablishmentsBlock } from '#/pages/cursus/catalog/ui/catalog-blocks'

function renderBlock(canManage: boolean) {
	render(
		<EstablishmentsBlock
			items={[
				{
					id: 'e-1',
					organization_id: 'org-1',
					name: 'CFA du Bâtiment',
					uai: '0690001A',
					is_main_site: true,
					created_at: '2026-01-01T00:00:00Z',
					updated_at: '2026-01-01T00:00:00Z',
				},
			]}
			isLoading={false}
			error={null}
			canManage={canManage}
			editor={null}
			isSaving={false}
			saveError={null}
			onOpenEditor={vi.fn()}
			onCloseEditor={vi.fn()}
			onSave={vi.fn()}
			onArchive={vi.fn()}
		/>,
	)
}

describe('EstablishmentsBlock', () => {
	it('lists establishments with the write controls for a manager', () => {
		renderBlock(true)

		expect(screen.getByText('CFA du Bâtiment')).toBeTruthy()
		expect(screen.getByText('Site principal')).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Nouvel établissement' }),
		).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Archiver CFA du Bâtiment' }),
		).toBeTruthy()
	})

	it('hides every write control for a reader', () => {
		renderBlock(false)

		expect(screen.getByText('CFA du Bâtiment')).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: 'Nouvel établissement' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Archiver CFA du Bâtiment' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Modifier CFA du Bâtiment' }),
		).toBeNull()
	})
})
