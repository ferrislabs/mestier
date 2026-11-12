import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { ClosureReasonsBlock } from '#/pages/cursus/catalog/ui/catalog-blocks'

function renderBlock(canManage: boolean) {
	render(
		<ClosureReasonsBlock
			items={[
				{
					id: 'r-1',
					organization_id: 'org-1',
					label: 'Rupture',
					created_at: '2026-01-01T00:00:00Z',
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

describe('ClosureReasonsBlock', () => {
	it('lists closure reasons with the write controls for a manager', () => {
		renderBlock(true)

		expect(screen.getByText('Rupture')).toBeTruthy()
		expect(screen.getByRole('button', { name: 'Nouveau motif' })).toBeTruthy()
		expect(
			screen.getByRole('button', { name: 'Archiver Rupture' }),
		).toBeTruthy()
	})

	it('hides every write control for a reader', () => {
		renderBlock(false)

		expect(screen.getByText('Rupture')).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Nouveau motif' })).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Archiver Rupture' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Modifier Rupture' }),
		).toBeNull()
	})
})
