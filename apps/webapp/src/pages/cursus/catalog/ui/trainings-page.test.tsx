import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import {
	TrainingsPage,
	type TrainingsPageProps,
} from '#/pages/cursus/catalog/ui/trainings-page'

const TRAINING = {
	id: 't-1',
	organization_id: 'org-1',
	name: 'CAP Menuiserie',
	rncp_code: 'RNCP12345',
	level: '3',
	degree_type: 'CAP',
	archived_at: '2026-06-01T00:00:00Z',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

function renderPage(overrides: Partial<TrainingsPageProps> = {}) {
	render(
		<TrainingsPage
			items={[TRAINING]}
			total={1}
			search=""
			archived="all"
			isLoading={false}
			error={null}
			canManage
			editor={null}
			isSaving={false}
			saveError={null}
			onSearchChange={vi.fn()}
			onArchivedChange={vi.fn()}
			onResetFilters={vi.fn()}
			onOpenEditor={vi.fn()}
			onCloseEditor={vi.fn()}
			onSave={vi.fn()}
			onArchive={vi.fn()}
			{...overrides}
		/>,
	)
}

describe('TrainingsPage', () => {
	it('shows name, RNCP code, level, degree and status columns', () => {
		renderPage()

		expect(screen.getByText('CAP Menuiserie')).toBeTruthy()
		expect(screen.getByText('RNCP12345')).toBeTruthy()
		expect(screen.getByText('3')).toBeTruthy()
		expect(screen.getByText('CAP')).toBeTruthy()
		expect(screen.getByText('Archivée')).toBeTruthy()
		expect(screen.getByText('1 formation')).toBeTruthy()
	})

	it('has no row menu on an archived training', () => {
		renderPage()

		expect(
			screen.queryByRole('button', { name: 'Actions CAP Menuiserie' }),
		).toBeNull()
	})
})
