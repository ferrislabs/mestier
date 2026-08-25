import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { Project } from '#/hooks/use-projects'
import { ProjectsList } from '#/pages/projects/ui/projects-list'
import { renderWithRouter } from '#/test/render-with-router'

function project(overrides: Partial<Project> = {}): Project {
	return {
		id: 'project-1',
		organization_id: 'org-1',
		name: 'Jardin Duval',
		customer_id: 'customer-1',
		customer_context_id: null,
		quote_id: null,
		archived_at: null,
		is_internal: false,
		created_at: '2026-08-01T08:00:00Z',
		updated_at: '2026-08-01T08:00:00Z',
		...overrides,
	}
}

function baseProps() {
	return {
		organizationName: 'Atelier Vert',
		projects: [project()],
		customerName: () => 'Duval Maçonnerie',
		search: '',
		includeArchived: false,
		isLoading: false,
		error: null,
		editingId: null,
		isSaving: false,
		isBuildingTemplate: false,
		onSearchChange: vi.fn(),
		onIncludeArchivedChange: vi.fn(),
		onCreate: vi.fn(),
		onStartFromTemplate: vi.fn(),
		onEdit: vi.fn(),
		onCancelEdit: vi.fn(),
		onSaveEdit: vi.fn(),
		onArchive: vi.fn(),
		onRestore: vi.fn(),
		onSaveAsTemplate: vi.fn(),
	}
}

describe('ProjectsList', () => {
	it('names the customer rather than showing an id', async () => {
		await renderWithRouter(<ProjectsList {...baseProps()} />)

		expect(screen.getByText('Duval Maçonnerie')).toBeDefined()
		expect(screen.queryByText('customer-1')).toBeNull()
	})

	/**
	 * The case the entity exists for. A project with no customer must not read
	 * as a form somebody failed to finish.
	 */
	it('marks a customer-less project as internal, deliberately', async () => {
		await renderWithRouter(
			<ProjectsList
				{...baseProps()}
				projects={[
					project({
						id: 'project-internal',
						name: 'Réunion hebdo',
						customer_id: null,
						is_internal: true,
					}),
				]}
			/>,
		)

		expect(screen.getByText('Interne')).toBeDefined()
		expect(screen.getByText(/aucun, projet interne/i)).toBeDefined()
	})

	it('says a project without a quote has no margin', async () => {
		await renderWithRouter(<ProjectsList {...baseProps()} />)

		expect(screen.getByText(/aucun, pas de marge/i)).toBeDefined()
	})

	/** Archiving is reversible, so an archived row offers the way back. */
	it('offers a restore on an archived project instead of an edit', async () => {
		const onRestore = vi.fn()
		await renderWithRouter(
			<ProjectsList
				{...baseProps()}
				includeArchived
				projects={[project({ archived_at: '2026-08-10T08:00:00Z' })]}
				onRestore={onRestore}
			/>,
		)

		expect(screen.getByText('Archivé')).toBeDefined()
		expect(screen.getByRole('button', { name: /restaurer/i })).toBeDefined()
	})

	it('counts internal projects separately, since they are the new case', async () => {
		await renderWithRouter(
			<ProjectsList
				{...baseProps()}
				projects={[
					project(),
					project({ id: 'p2', customer_id: null, is_internal: true }),
					project({ id: 'p3', customer_id: null, is_internal: true }),
				]}
			/>,
		)

		expect(screen.getByText('Internes')).toBeDefined()
		expect(screen.getByText('2')).toBeDefined()
	})

	it('says so plainly when there is nothing yet', async () => {
		await renderWithRouter(<ProjectsList {...baseProps()} projects={[]} />)

		expect(screen.getByText(/aucun projet/i)).toBeDefined()
	})
})
