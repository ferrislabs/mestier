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
		customerContexts: [{ id: 'ctx-1', label: 'Chantier principal' }],
		isCustomerContextsLoading: false,
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
})

describe('TaskFormFields — subtask', () => {
	it('shows the inherited-window placeholder on the date fields when provided', () => {
		render(
			<TaskFormFields
				{...baseProps()}
				isSubtask={true}
				windowPlaceholder="Hérite du parent : 10/08/2026 09:00 – 11:00"
			/>,
		)
		expect(
			screen.getAllByPlaceholderText(
				'Hérite du parent : 10/08/2026 09:00 – 11:00',
			).length,
		).toBe(2)
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
