import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { emptyTaskDraft } from '#/pages/planning/lib/task-form'
import { TaskFormFields } from '#/pages/planning/ui/task-form-fields'

class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
;(globalThis as any).ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

const LABELS = [{ id: 'l1', name: 'Réunion', color: '#2563EB' }]
const ASSIGNEE_OPTIONS = [
	{ resourceId: 'member:e1', displayName: 'Alix Martin' },
]
const CUSTOMERS = [{ id: 'cust-1', displayName: 'Dupont Alice' }]

function baseProps() {
	return {
		mode: 'create' as const,
		isSubtask: false,
		values: emptyTaskDraft({ parentTaskId: null, today: '2026-08-10' }),
		onChange: vi.fn(),
		errors: [] as string[],
		windowPlaceholder: null as string | null,
		customerName: null as string | null,
		customers: CUSTOMERS,
		customerContexts: [{ id: 'ctx-1', label: 'Projet principal' }],
		isCustomerContextsLoading: false,
		quotes: [{ id: 'quote-1', label: 'DEV-2026-0001 · 4 200,00 €' }],
		isQuotesLoading: false,
		labels: LABELS,
		isCreatingLabel: false,
		onCreateLabel: vi.fn(),
		assigneeOptions: ASSIGNEE_OPTIONS,
	}
}

describe('TaskFormFields — champs de base', () => {
	it('reports title changes through onChange', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<TaskFormFields {...baseProps()} onChange={onChange} />)

		await user.type(screen.getByLabelText('Titre'), 'x')
		expect(onChange).toHaveBeenCalled()
	})

	it('hides the time inputs when all-day is on', () => {
		render(
			<TaskFormFields
				{...baseProps()}
				values={{ ...baseProps().values, allDay: true }}
			/>,
		)
		expect(screen.queryByLabelText('Heure de début')).toBeNull()
	})

	it('shows the time inputs when all-day is off', () => {
		render(<TaskFormFields {...baseProps()} />)
		expect(screen.getByLabelText('Heure de début')).toBeDefined()
	})

	it('always renders the blocks_availability question, whatever its value', () => {
		render(
			<TaskFormFields
				{...baseProps()}
				values={{ ...baseProps().values, blocksAvailability: false }}
			/>,
		)
		expect(
			screen.getByText(/rend-elle la personne indisponible/i),
		).toBeDefined()
	})
})

describe('TaskFormFields — client', () => {
	it('offers a customer selector in create mode', () => {
		render(<TaskFormFields {...baseProps()} mode="create" />)
		expect(screen.getByLabelText('Client')).toBeDefined()
	})

	it('shows the customer as static text in edit mode, never a selector', () => {
		render(
			<TaskFormFields
				{...baseProps()}
				mode="edit"
				customerName="Dupont Alice"
			/>,
		)
		expect(screen.queryByLabelText('Client')).toBeNull()
		expect(screen.getByText('Dupont Alice')).toBeDefined()
	})

	it('shows "aucun client" in edit mode when the task has none — a réunion is not a workaround', () => {
		render(<TaskFormFields {...baseProps()} mode="edit" customerName={null} />)
		expect(screen.getByText(/Aucun client/)).toBeDefined()
	})

	it('offers an explicit "aucun" choice back to no customer', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<TaskFormFields
				{...baseProps()}
				onChange={onChange}
				values={{ ...baseProps().values, customerId: 'cust-1' }}
			/>,
		)

		await user.click(screen.getByLabelText('Client'))
		await user.click(
			screen.getByRole('option', { name: /Aucun — réunion, déplacement/ }),
		)

		expect(onChange).toHaveBeenCalledWith({
			customerId: '',
			customerContextId: '',
		})
	})

	it('a context is optional even once a customer is picked', async () => {
		const user = userEvent.setup()
		render(
			<TaskFormFields
				{...baseProps()}
				values={{ ...baseProps().values, customerId: 'cust-1' }}
			/>,
		)

		expect(screen.getByLabelText('Contexte')).toBeDefined()

		await user.click(screen.getByLabelText('Contexte'))
		expect(
			screen.getByRole('option', { name: 'Aucun contexte précis' }),
		).toBeDefined()
	})
})

describe('TaskFormFields — subtask', () => {
	it('shows the inherited-window placeholder as the range trigger label when the subtask has no dates of its own', () => {
		render(
			<TaskFormFields
				{...baseProps()}
				isSubtask={true}
				windowPlaceholder="Hérite du parent : 10/08/2026 09:00 – 11:00"
				values={{ ...baseProps().values, startDate: '', endDate: '' }}
			/>,
		)
		expect(
			screen.getByRole('button', {
				name: 'Hérite du parent : 10/08/2026 09:00 – 11:00',
			}),
		).toBeDefined()
	})

	it('shows the actual range, not the placeholder, once the subtask has its own dates', () => {
		render(
			<TaskFormFields
				{...baseProps()}
				isSubtask={true}
				windowPlaceholder="Hérite du parent : 10/08/2026 09:00 – 11:00"
			/>,
		)
		expect(
			screen.queryByRole('button', { name: /Hérite du parent/ }),
		).toBeNull()
	})
})

describe('TaskFormFields — erreurs', () => {
	it('renders validation errors', () => {
		render(<TaskFormFields {...baseProps()} errors={['Titre requis']} />)
		expect(screen.getByText('Titre requis')).toBeDefined()
	})
})

describe('TaskFormFields — no network call', () => {
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

	it('never triggers fetch at render', () => {
		render(<TaskFormFields {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})

describe('TaskFormFields — devis', () => {
	/**
	 * Without this link the profitability report can state a projet's cost but
	 * never its margin, which is the whole point of M6. The field is the only
	 * place in the product that sets `tasks.quote_id`.
	 */
	it('offers the customer quotes once a customer is chosen', async () => {
		const props = baseProps()
		render(
			<TaskFormFields
				{...props}
				values={{ ...props.values, customerId: CUSTOMERS[0].id }}
			/>,
		)

		expect(screen.getByLabelText('Devis')).toBeDefined()
	})

	it('says what is lost by leaving it empty', async () => {
		const props = baseProps()
		render(
			<TaskFormFields
				{...props}
				values={{ ...props.values, customerId: CUSTOMERS[0].id }}
			/>,
		)

		expect(screen.getByText(/aucune marge/i)).toBeDefined()
	})

	/** A quote belongs to a customer, so there is nothing to choose from yet. */
	it('is absent until a customer is chosen', async () => {
		const props = baseProps()
		render(
			<TaskFormFields
				{...props}
				values={{ ...props.values, customerId: '' }}
			/>,
		)

		expect(screen.queryByLabelText('Devis')).toBeNull()
	})

	/**
	 * Edit mode shows no quote selector, for the same reason it shows no customer
	 * selector: `PATCH /tasks/{id}` carries neither field.
	 */
	it('is absent in edit mode', async () => {
		const props = baseProps()
		render(
			<TaskFormFields
				{...props}
				mode="edit"
				values={{ ...props.values, customerId: CUSTOMERS[0].id }}
			/>,
		)

		expect(screen.queryByLabelText('Devis')).toBeNull()
	})
})
