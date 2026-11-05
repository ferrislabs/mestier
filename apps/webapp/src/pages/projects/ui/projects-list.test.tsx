import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { Project } from '#/hooks/use-projects'
import { EMPTY_EXTRAS } from '#/pages/projects/lib/model'
import {
	DEFAULT_FILTERS,
	type ProjectRow,
} from '#/pages/projects/lib/project-list'
import {
	ProjectsList,
	type ProjectsListProps,
} from '#/pages/projects/ui/projects-list'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
const globalAny = globalThis as any
globalAny.ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false

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

function row(overrides: Partial<ProjectRow> = {}): ProjectRow {
	return {
		project: project(),
		status: 'active',
		customerName: 'Duval Maçonnerie',
		extras: EMPTY_EXTRAS,
		missions: { total: 0, todo: 0, inProgress: 0, done: 0, progress: 0 },
		doeRevision: null,
		owner: undefined,
		...overrides,
	}
}

function baseProps(): ProjectsListProps {
	return {
		organizationSlug: 'atelier-vert',
		summary: '1 projet actif, 0 archivé',
		tiles: { active: 1, internal: 0, missionsInProgress: 0, doeToCreate: 1 },
		filters: DEFAULT_FILTERS,
		customers: [],
		listed: [row()],
		empty: 'none',
		newProjectCustomerId: undefined,
		isLoading: false,
		error: null,
		onFiltersChange: vi.fn(),
		onStartFromTemplate: vi.fn(),
	}
}

describe('ProjectsList', () => {
	it('names the customer rather than showing an id', async () => {
		await renderWithRouter(
			wrapWithPermissions(<ProjectsList {...baseProps()} />),
		)

		expect(screen.getByText('Duval Maçonnerie')).toBeDefined()
		expect(screen.queryByText('customer-1')).toBeNull()
	})

	it('marks a customer-less project as internal, deliberately', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<ProjectsList
					{...baseProps()}
					listed={[
						row({
							project: project({
								id: 'project-internal',
								name: 'Réunion hebdo',
								customer_id: null,
								is_internal: true,
							}),
							customerName: null,
						}),
					]}
				/>,
			),
		)

		expect(screen.getByText('Projet interne')).toBeDefined()
	})

	it('shows an archived project as such', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<ProjectsList
					{...baseProps()}
					listed={[
						row({
							project: project({ archived_at: '2026-08-10T08:00:00Z' }),
							status: 'archived',
						}),
					]}
				/>,
			),
		)

		expect(screen.getAllByText('Archivé').length).toBeGreaterThan(0)
	})

	it('counts internal projects in the first tile', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<ProjectsList
					{...baseProps()}
					tiles={{
						active: 3,
						internal: 2,
						missionsInProgress: 0,
						doeToCreate: 0,
					}}
				/>,
			),
		)

		expect(screen.getByText('dont 2 en interne')).toBeDefined()
	})

	it('says so plainly when there is nothing yet', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<ProjectsList {...baseProps()} listed={[]} empty="none" />,
			),
		)

		expect(screen.getByText('Aucun projet pour l’instant')).toBeDefined()
	})

	it('offers to reset the filters when none match', async () => {
		const onFiltersChange = vi.fn()
		await renderWithRouter(
			wrapWithPermissions(
				<ProjectsList
					{...baseProps()}
					filters={{ ...DEFAULT_FILTERS, query: 'zzz' }}
					listed={[]}
					empty="no-match"
					onFiltersChange={onFiltersChange}
				/>,
			),
		)

		screen.getByRole('button', { name: 'Réinitialiser les filtres' }).click()

		expect(onFiltersChange).toHaveBeenCalledWith(DEFAULT_FILTERS)
	})

	it('opens the project from its name', async () => {
		await renderWithRouter(
			wrapWithPermissions(<ProjectsList {...baseProps()} />),
		)

		expect(
			screen.getByRole('link', { name: /Jardin Duval/ }).getAttribute('href'),
		).toBe('/o/atelier-vert/projet/project-1')
	})

	it('shows the state of each project DOE', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<ProjectsList
					{...baseProps()}
					listed={[
						row({ doeRevision: 'B' }),
						row({
							project: project({ id: 'p2', name: 'Parc de Vichy' }),
						}),
					]}
				/>,
			),
		)

		expect(screen.getByText('Indice B')).toBeDefined()
		expect(screen.getByText('Aucun')).toBeDefined()
	})
})
