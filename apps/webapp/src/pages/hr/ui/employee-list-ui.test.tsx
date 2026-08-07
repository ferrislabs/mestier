import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Employee } from '#/hooks/use-reference-catalog'
import { EmployeeListUI } from '#/pages/hr/ui/employee-list-ui'
import { renderWithRouter } from '#/test/render-with-router'

function employee(overrides: Partial<Employee> = {}): Employee {
	return {
		id: 'employee-1',
		organization_id: 'org-1',
		last_name: 'Martin',
		first_name: 'Alix',
		hourly_rate_cents: 1500,
		user_id: null,
		weekly_contract_minutes: 2100,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function baseProps() {
	return {
		organizationName: 'Atelier Bois & Co',
		isLoading: false,
		error: null,
		data: { employees: [employee()] },
		search: '',
		onSearchChange: vi.fn(),
		createForm: {
			values: { lastName: '', firstName: '', hourlyRate: '', userId: '' },
			isPending: false,
			onChange: vi.fn(),
			onSubmit: vi.fn(),
		},
		draft: null,
		isSaving: false,
		onEdit: vi.fn(),
		onDraftChange: vi.fn(),
		onCancelEdit: vi.fn(),
		onSaveEdit: vi.fn(),
		onDeleteEmployee: vi.fn(),
	}
}

describe('EmployeeListUI — rendu du nom', () => {
	it('affiche « {nom} {prénom} » quand le prénom est renseigné', async () => {
		await renderWithRouter(<EmployeeListUI {...baseProps()} />)

		expect(screen.getByText('Martin Alix')).toBeDefined()
	})

	it('affiche le seul nom, sans espace résiduel, quand le prénom est absent', async () => {
		await renderWithRouter(
			<EmployeeListUI
				{...baseProps()}
				data={{ employees: [employee({ first_name: null })] }}
			/>,
		)

		expect(screen.getByText('Martin')).toBeDefined()
		expect(screen.queryByText('Martin Alix')).toBeNull()
	})
})

describe('EmployeeListUI — accès à l’écran de temps de travail', () => {
	it('expose un lien vers le temps de travail de l’employé, pas seulement un rendu', async () => {
		const user = userEvent.setup()
		await renderWithRouter(<EmployeeListUI {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		// Radix's `asChild` merges the anchor into the menu item and overrides
		// its role to `menuitem` — it is still a real `<a href>` underneath.
		const link = screen.getByRole('menuitem', { name: /Temps de travail/ })
		expect(link.tagName).toBe('A')
		expect(link.getAttribute('href')).toBe('/hr/employees/employee-1/work-time')
	})
})
