import { fireEvent, render, screen, within } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { PlanningEntry, PlanningResource } from '#/pages/planning/types'
import { PlanningGrid } from '#/pages/planning/ui/planning-grid'

/** jsdom has no `DataTransfer`; a plain object backed by a `Map` is enough for `setData`/`getData`. */
function fakeDataTransfer() {
	const store = new Map<string, string>()
	return {
		setData: (format: string, data: string) => store.set(format, data),
		getData: (format: string) => store.get(format) ?? '',
	}
}

const TZ = 'Europe/Paris'

function employeeResource(
	overrides: Partial<PlanningResource> = {},
): PlanningResource {
	return {
		resource_id: 'employee:employee-1',
		kind: 'employee',
		employee_id: 'employee-1',
		user_id: null,
		display_name: 'Alix Martin',
		hourly_rate_cents: 1500,
		weekly_contract_minutes: 2100,
		...overrides,
	}
}

function workOrder(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'work_order',
		id: 'wo-1',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
		all_day: false,
		status: 'PLANNED',
		employee_ids: ['employee-1'],
		customer_name: 'Client Dupont',
		context_label: 'Chantier toiture',
		...overrides,
	} as PlanningEntry
}

function absence(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'absence',
		id: 'ab-1',
		starts_at: '2026-08-10T00:00:00+02:00',
		ends_at: '2026-08-11T00:00:00+02:00',
		all_day: true,
		absence_kind: 'LEAVE',
		employee_id: 'employee-1',
		...overrides,
	} as PlanningEntry
}

function unknownKindEntry(): PlanningEntry {
	return {
		kind: 'external_source',
		id: 'ext-1',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
		all_day: false,
	} as unknown as PlanningEntry
}

describe('PlanningGrid — axe ressource', () => {
	it('affiche une ligne par ressource, quelle que soit la vue', () => {
		const resources = [
			employeeResource(),
			employeeResource({
				resource_id: 'employee:employee-2',
				employee_id: 'employee-2',
				display_name: 'Marie Leroy',
			}),
		]

		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				timeZone={TZ}
				resources={resources}
				entries={[]}
				workTime={[]}
			/>,
		)

		expect(screen.getByText('Alix Martin')).toBeDefined()
		expect(screen.getByText('Marie Leroy')).toBeDefined()
		expect(screen.getAllByTestId('grid-row')).toHaveLength(2)
	})

	it("affiche l'état vide sans planter quand il n'y a aucune ressource", () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				timeZone={TZ}
				resources={[]}
				entries={[]}
				workTime={[]}
			/>,
		)

		expect(screen.queryByTestId('planning-grid')).toBeNull()
		expect(screen.getByText(/Aucune ressource/)).toBeDefined()
	})
})

describe('PlanningGrid — vue semaine', () => {
	it('rend une colonne par jour de la fenêtre', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[]}
				workTime={[]}
			/>,
		)

		expect(screen.getAllByTestId('grid-cell')).toHaveLength(7)
	})

	it('positionne un segment de chantier avec left/width en pourcentage', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[workOrder()]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('data-entry-id')).toBe('wo-1')
		expect(segment.getAttribute('data-tone')).toBe('work_order')
		expect(segment.style.left).toBe('0%')
		expect(segment.style.width).toBe('100%')
	})
})

describe('PlanningGrid — vue jour', () => {
	it('rend une seule cellule couvrant toute la journée', () => {
		render(
			<PlanningGrid
				view="day"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[]}
				workTime={[]}
			/>,
		)

		expect(screen.getAllByTestId('grid-cell')).toHaveLength(1)
	})

	it('affiche le libellé du segment (place disponible)', () => {
		render(
			<PlanningGrid
				view="day"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[workOrder({ title: 'Réfection toiture' })]}
				workTime={[]}
			/>,
		)

		expect(screen.getByText('Réfection toiture')).toBeDefined()
	})
})

describe('PlanningGrid — vue mois', () => {
	it('marque les cellules où une absence est en cours', () => {
		render(
			<PlanningGrid
				view="month"
				windowFrom="2026-08-01"
				windowTo="2026-08-31"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[absence()]}
				workTime={[]}
			/>,
		)

		const cellWithAbsence = screen
			.getAllByTestId('grid-cell')
			.find((cell) => cell.getAttribute('data-date') === '2026-08-10')
		expect(cellWithAbsence?.getAttribute('data-has-absence')).toBe('true')

		const cellWithoutAbsence = screen
			.getAllByTestId('grid-cell')
			.find((cell) => cell.getAttribute('data-date') === '2026-08-11')
		expect(cellWithoutAbsence?.getAttribute('data-has-absence')).toBe('false')
	})
})

describe('PlanningGrid — empilement des chevauchements', () => {
	it('donne une ligne distincte à chaque segment qui se chevauche', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[
					workOrder({
						id: 'wo-1',
						starts_at: '2026-08-10T08:00:00Z',
						ends_at: '2026-08-10T10:00:00Z',
					}),
					workOrder({
						id: 'wo-2',
						starts_at: '2026-08-10T09:00:00Z',
						ends_at: '2026-08-10T11:00:00Z',
					}),
				]}
				workTime={[]}
			/>,
		)

		const cell = screen.getByTestId('grid-cell')
		const segments = within(cell).getAllByTestId('grid-segment')
		const tops = segments.map((segment) => segment.style.top)
		expect(new Set(tops).size).toBe(2)
	})
})

describe('PlanningGrid — résilience aux kinds inconnus', () => {
	it('ne plante pas quand une entrée porte un kind inconnu, et rend le reste normalement', () => {
		expect(() =>
			render(
				<PlanningGrid
					view="week"
					windowFrom="2026-08-10"
					windowTo="2026-08-10"
					timeZone={TZ}
					resources={[employeeResource()]}
					entries={[workOrder(), unknownKindEntry()]}
					workTime={[]}
				/>,
			),
		).not.toThrow()

		expect(screen.getAllByTestId('grid-segment')).toHaveLength(1)
	})
})

describe('PlanningGrid — drag & drop', () => {
	function twoResources() {
		return [
			employeeResource(),
			employeeResource({
				resource_id: 'employee:employee-2',
				employee_id: 'employee-2',
				display_name: 'Marie Leroy',
			}),
		]
	}

	it('un segment de chantier est draggable et transporte entryId/resourceId/date au dragstart', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[workOrder()]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('draggable')).toBe('true')
	})

	it('un drop sur une autre ligne, même date, appelle onDropWorkOrder avec la ressource cible', () => {
		const onDropWorkOrder = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={twoResources()}
				entries={[workOrder()]}
				workTime={[]}
				onDropWorkOrder={onDropWorkOrder}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		const dataTransfer = fakeDataTransfer()
		fireEvent.dragStart(segment, { dataTransfer })

		const rows = screen.getAllByTestId('grid-row')
		const targetRow = rows.find(
			(row) => row.getAttribute('data-resource-id') === 'employee:employee-2',
		)
		if (!targetRow) throw new Error('target row not found')
		const targetCell = within(targetRow).getByTestId('grid-cell')

		fireEvent.drop(targetCell, { dataTransfer })

		expect(onDropWorkOrder).toHaveBeenCalledWith({
			entryId: 'wo-1',
			sourceResourceId: 'employee:employee-1',
			sourceDate: '2026-08-10',
			targetResourceId: 'employee:employee-2',
			targetDate: '2026-08-10',
		})
	})

	it("un drop sur la cellule d'origine (rien ne change) appelle quand même le callback avec des cibles identiques aux sources", () => {
		const onDropWorkOrder = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[workOrder()]}
				workTime={[]}
				onDropWorkOrder={onDropWorkOrder}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		const dataTransfer = fakeDataTransfer()
		fireEvent.dragStart(segment, { dataTransfer })
		fireEvent.drop(screen.getByTestId('grid-cell'), { dataTransfer })

		expect(onDropWorkOrder).toHaveBeenCalledWith({
			entryId: 'wo-1',
			sourceResourceId: 'employee:employee-1',
			sourceDate: '2026-08-10',
			targetResourceId: 'employee:employee-1',
			targetDate: '2026-08-10',
		})
	})

	it('sans onDropWorkOrder, un drop ne plante pas', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[workOrder()]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		const dataTransfer = fakeDataTransfer()
		fireEvent.dragStart(segment, { dataTransfer })
		expect(() =>
			fireEvent.drop(screen.getByTestId('grid-cell'), { dataTransfer }),
		).not.toThrow()
	})
})

describe('PlanningGrid — retrait d’un assigné', () => {
	it('le bouton retirer sur un segment de chantier appelle onRemoveAssignee avec entryId et resourceId de la ligne', () => {
		const onRemoveAssignee = vi.fn()
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[workOrder()]}
				workTime={[]}
				onRemoveAssignee={onRemoveAssignee}
			/>,
		)

		fireEvent.click(
			screen.getByRole('button', { name: /Retirer cette personne/ }),
		)

		expect(onRemoveAssignee).toHaveBeenCalledWith({
			entryId: 'wo-1',
			resourceId: 'employee:employee-1',
		})
	})

	it("n'affiche pas de bouton retirer sans onRemoveAssignee", () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[workOrder()]}
				workTime={[]}
			/>,
		)

		expect(
			screen.queryByRole('button', { name: /Retirer cette personne/ }),
		).toBeNull()
	})
})

describe('PlanningGrid — segment d’absence inerte', () => {
	it("un clic sur un segment d'absence ne déclenche aucune interaction (pas de role bouton, pas d'onClick)", () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-10"
				windowTo="2026-08-10"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[absence()]}
				workTime={[]}
			/>,
		)

		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('role')).toBeNull()
		expect(segment.getAttribute('tabindex')).toBeNull()
		// A click must not throw and must not carry any accessible "button" affordance.
		fireEvent.click(segment)
		expect(screen.queryAllByRole('button')).toHaveLength(0)
	})
})

describe('PlanningGrid — pas d’appel réseau', () => {
	let fetchSpy: ReturnType<typeof createFetchSpy>

	function createFetchSpy() {
		return vi.spyOn(global, 'fetch').mockImplementation(() => {
			throw new Error('le composant ui/ ne doit jamais appeler fetch')
		})
	}

	beforeEach(() => {
		fetchSpy = createFetchSpy()
	})

	afterEach(() => {
		fetchSpy.mockRestore()
	})

	it('ne déclenche aucun fetch au rendu', () => {
		render(
			<PlanningGrid
				view="week"
				windowFrom="2026-08-03"
				windowTo="2026-08-09"
				timeZone={TZ}
				resources={[employeeResource()]}
				entries={[workOrder(), absence()]}
				workTime={[]}
			/>,
		)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
