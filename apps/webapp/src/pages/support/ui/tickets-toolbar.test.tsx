import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'
import type { SupportToolbarProps } from '#/pages/support/types'
import { ALIX, JO } from '#/pages/support/ui/support-fixtures'
import { TicketListHeader } from '#/pages/support/ui/ticket-list-header'
import { TicketsToolbar } from '#/pages/support/ui/tickets-toolbar'

beforeAll(installRadixPolyfills)

function props(
	overrides: Partial<SupportToolbarProps> = {},
): SupportToolbarProps {
	return {
		title: 'Boîte de réception',
		count: 3,
		query: '',
		onQueryChange: vi.fn(),
		filters: {},
		members: [ALIX, JO],
		assigneeCounts: new Map([
			['member-1', 2],
			[null, 1],
		]),
		badges: [{ id: 'Urgent', label: 'Urgent' }],
		fieldKeys: ['ville'],
		isLoading: false,
		showStatus: true,
		view: 'board',
		onViewChange: vi.fn(),
		displayFieldOptions: ['pays', 'ville'],
		displayFields: ['ville'],
		canEditDisplayFields: true,
		onDisplayFieldsChange: vi.fn(),
		onChange: vi.fn(),
		onReset: vi.fn(),
		...overrides,
	}
}

describe('TicketsToolbar', () => {
	it('reports what is typed in the search field', async () => {
		const toolbar = props()
		render(<TicketsToolbar {...toolbar} />)

		await userEvent.type(screen.getByLabelText('Rechercher un ticket'), 'f')

		expect(toolbar.onQueryChange).toHaveBeenCalledWith('f')
	})

	it('switches the view from the toggle', async () => {
		const toolbar = props()
		render(<TicketsToolbar {...toolbar} />)

		await userEvent.click(screen.getByRole('radio', { name: 'Liste' }))

		expect(toolbar.onViewChange).toHaveBeenCalledWith('list')
	})

	it('marks the current view as selected', () => {
		render(<TicketsToolbar {...props({ view: 'list' })} />)

		expect(
			screen.getByRole('radio', { name: 'Liste' }).getAttribute('aria-checked'),
		).toBe('true')
	})

	it('shows no counter on the filters button without active filter', () => {
		render(<TicketsToolbar {...props()} />)

		expect(screen.queryByRole('img')).toBeNull()
	})

	it('counts the active filters, the custom field pair included', () => {
		render(
			<TicketsToolbar
				{...props({
					filters: {
						badge: 'Urgent',
						custom_field_key: 'ville',
						custom_field_value: 'Lyon',
					},
				})}
			/>,
		)

		expect(screen.getByRole('img', { name: '2 filtres actifs' })).toBeTruthy()
	})

	it('uses the singular for one active filter', () => {
		render(<TicketsToolbar {...props({ filters: { priority: 'low' } })} />)

		expect(screen.getByRole('img', { name: '1 filtre actif' })).toBeTruthy()
	})

	it('hides the status filter in the board toolbar', async () => {
		render(<TicketsToolbar {...props({ showStatus: false })} />)

		await userEvent.click(screen.getByRole('button', { name: /Filtres/ }))

		expect(await screen.findByText('Priorité')).toBeTruthy()
		expect(screen.queryByText('Statut')).toBeNull()
	})

	it('resets from the popover once a filter is active', async () => {
		const toolbar = props({ filters: { badge: 'Urgent' } })
		render(<TicketsToolbar {...toolbar} />)

		await userEvent.click(screen.getByRole('button', { name: /Filtres/ }))
		await userEvent.click(
			await screen.findByRole('button', { name: /Réinitialiser/ }),
		)

		expect(toolbar.onReset).toHaveBeenCalledTimes(1)
	})

	it('disables the reset without active filter', async () => {
		render(<TicketsToolbar {...props()} />)

		await userEvent.click(screen.getByRole('button', { name: /Filtres/ }))

		expect(
			(
				(await screen.findByRole('button', {
					name: /Réinitialiser/,
				})) as HTMLButtonElement
			).disabled,
		).toBe(true)
	})

	it('offers the members with their open ticket count in the assignee filter', async () => {
		const toolbar = props()
		render(<TicketsToolbar {...toolbar} />)

		await userEvent.click(screen.getByRole('button', { name: /Filtres/ }))
		await userEvent.click(await screen.findByLabelText('Assigné à'))

		expect(
			(await screen.findAllByRole('option')).map(
				(option) => option.textContent,
			),
		).toEqual([
			'Tout le monde',
			'Non assignés1',
			'AMAlix Martin2',
			'JDJo Durand0',
		])
	})

	it('writes the unassigned choice as the "none" assignee', async () => {
		const toolbar = props()
		render(<TicketsToolbar {...toolbar} />)

		await userEvent.click(screen.getByRole('button', { name: /Filtres/ }))
		await userEvent.click(await screen.findByLabelText('Assigné à'))
		await userEvent.click(
			await screen.findByRole('option', { name: /Non assignés/ }),
		)

		expect(toolbar.onChange).toHaveBeenCalledWith({ assignee: 'none' })
	})

	it('writes the field value on Enter and ignores an unchanged one', async () => {
		const toolbar = props({
			filters: { custom_field_key: 'ville', custom_field_value: 'Lyon' },
		})
		render(<TicketsToolbar {...toolbar} />)
		await userEvent.click(screen.getByRole('button', { name: /Filtres/ }))
		const value = await screen.findByLabelText('Valeur')

		await userEvent.type(value, '{Enter}')
		expect(toolbar.onChange).not.toHaveBeenCalled()

		await userEvent.clear(value)
		await userEvent.type(value, 'Paris{Enter}')

		expect(toolbar.onChange).toHaveBeenCalledWith({
			custom_field_value: 'Paris',
		})
	})

	it('locks the field value until a field key is picked', async () => {
		render(<TicketsToolbar {...props()} />)

		await userEvent.click(screen.getByRole('button', { name: /Filtres/ }))

		expect(
			((await screen.findByLabelText('Valeur')) as HTMLInputElement).disabled,
		).toBe(true)
	})
})

describe('display fields menu', () => {
	it('checks the displayed fields and saves a toggled one', async () => {
		const toolbar = props()
		render(<TicketsToolbar {...toolbar} />)

		await userEvent.click(
			screen.getByRole('button', { name: 'Champs affichés' }),
		)

		expect(
			(
				await screen.findByRole('menuitemcheckbox', { name: 'ville' })
			).getAttribute('aria-checked'),
		).toBe('true')
		await userEvent.click(
			screen.getByRole('menuitemcheckbox', { name: 'pays' }),
		)

		expect(toolbar.onDisplayFieldsChange).toHaveBeenCalledWith([
			'pays',
			'ville',
		])
	})

	it('says no field has been received yet', async () => {
		render(
			<TicketsToolbar
				{...props({ displayFieldOptions: [], displayFields: [] })}
			/>,
		)

		await userEvent.click(
			screen.getByRole('button', { name: 'Champs affichés' }),
		)

		expect(
			await screen.findByText('Aucun champ transmis pour le moment.'),
		).toBeTruthy()
	})

	it('locks the choices for a reader', async () => {
		render(<TicketsToolbar {...props({ canEditDisplayFields: false })} />)

		await userEvent.click(
			screen.getByRole('button', { name: 'Champs affichés' }),
		)

		expect(
			(
				await screen.findByRole('menuitemcheckbox', { name: 'pays' })
			).getAttribute('aria-disabled'),
		).toBe('true')
	})
})

describe('TicketListHeader', () => {
	it('shows the title and the number of displayed tickets', () => {
		render(<TicketListHeader {...props({ view: 'list' })} />)

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'Boîte de réception',
		)
		expect(screen.getByTitle(/Tickets affichés/).textContent).toBe('3')
	})

	it('carries the same search, filters and view controls', async () => {
		const header = props({ view: 'list' })
		render(<TicketListHeader {...header} />)

		await userEvent.type(screen.getByLabelText('Rechercher un ticket'), 'x')
		await userEvent.click(screen.getByRole('radio', { name: 'Kanban' }))

		expect(header.onQueryChange).toHaveBeenCalledWith('x')
		expect(header.onViewChange).toHaveBeenCalledWith('board')
		expect(screen.getByRole('button', { name: /Filtres/ })).toBeTruthy()
		expect(screen.getByRole('button', { name: 'Champs affichés' })).toBeTruthy()
	})
})
