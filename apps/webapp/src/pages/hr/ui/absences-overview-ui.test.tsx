import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { emptyAbsenceDraft } from '#/pages/hr/lib/absences'
import type {
	AbsenceOverviewRow,
	AbsencesOverviewUIProps,
} from '#/pages/hr/ui/absences-overview-ui'
import { AbsencesOverviewUI } from '#/pages/hr/ui/absences-overview-ui'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

function absence(
	overrides: Partial<AbsenceOverviewRow> = {},
): AbsenceOverviewRow {
	return {
		id: 'ab-1',
		memberDisplayName: 'Martin Alix',
		memberId: 'member-1',
		kind: 'LEAVE',
		allDay: true,
		range: { from: '2026-08-10', to: '2026-08-11' },
		startTime: '08:00',
		endTime: '18:00',
		note: '',
		...overrides,
	}
}

function baseProps(
	overrides: Partial<AbsencesOverviewUIProps> = {},
): AbsencesOverviewUIProps {
	return {
		organizationName: 'Atelier Bois & Co',
		isLoading: false,
		error: null,
		absences: [absence()],
		onCreate: vi.fn(),
		onEdit: vi.fn(),
		onDelete: vi.fn().mockResolvedValue(undefined),
		absenceSheet: {
			open: false,
			mode: 'create',
			values: emptyAbsenceDraft('', '2026-08-10'),
			members: [],
			errors: [],
			isSaving: false,
			isDeleting: false,
			saveError: null,
			onChange: vi.fn(),
			onSubmit: vi.fn(),
			onOpenChange: vi.fn(),
		},
		...overrides,
	}
}

describe('AbsencesOverviewUI — listing', () => {
	it('shows every absence with the resolved member name, nature and period', async () => {
		await renderWithRouter(
			wrapWithPermissions(<AbsencesOverviewUI {...baseProps()} />),
		)

		expect(screen.getByText('Martin Alix')).toBeDefined()
		expect(screen.getByText('Congé')).toBeDefined()
		expect(screen.getByText('Absences (1)')).toBeDefined()
	})

	it('shows an empty state when there is no absence', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<AbsencesOverviewUI {...baseProps({ absences: [] })} />,
			),
		)

		expect(screen.getByText('Aucune absence trouvée')).toBeDefined()
	})

	it('shows a loading placeholder instead of the table', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<AbsencesOverviewUI {...baseProps({ isLoading: true })} />,
			),
		)

		expect(screen.getByText('Chargement des absences…')).toBeDefined()
		expect(screen.queryByText('Martin Alix')).toBeNull()
	})

	it('surfaces a load error', async () => {
		await renderWithRouter(
			wrapWithPermissions(
				<AbsencesOverviewUI
					{...baseProps({ error: 'Impossible de charger' })}
				/>,
			),
		)

		expect(screen.getByText('Impossible de charger')).toBeDefined()
	})
})

describe('AbsencesOverviewUI — create action', () => {
	it('calls onCreate from the page header button', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(<AbsencesOverviewUI {...props} />),
		)

		await user.click(
			screen.getByRole('button', { name: /Ajouter une absence/ }),
		)

		expect(props.onCreate).toHaveBeenCalledTimes(1)
	})
})

describe('AbsencesOverviewUI — row actions', () => {
	it('calls onEdit with the row id from the row menu', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(<AbsencesOverviewUI {...props} />),
		)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Modifier/ }))

		expect(props.onEdit).toHaveBeenCalledWith('ab-1')
	})

	it('does not call onDelete until the confirmation dialog is accepted', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(
			wrapWithPermissions(<AbsencesOverviewUI {...props} />),
		)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Supprimer/ }))

		expect(props.onDelete).not.toHaveBeenCalled()
		expect(
			screen.getByRole('alertdialog', { name: /Supprimer l’absence/ }),
		).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => expect(props.onDelete).toHaveBeenCalledWith('ab-1'))
	})
})
