import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { emptyTaskDraft } from '#/pages/planning/lib/task-form'
import { TaskSheet } from '#/pages/planning/ui/task-sheet'

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

function fieldsProps() {
	return {
		mode: 'create' as const,
		isSubtask: false,
		values: emptyTaskDraft({ parentTaskId: null, today: '2026-08-10' }),
		onChange: vi.fn(),
		errors: [] as string[],
		windowPlaceholder: null,
		customerName: null,
		customers: [],
		customerContexts: [],
		quotes: [],
		projects: [],
		labels: [],
		onCreateLabel: vi.fn(),
		assigneeOptions: [],
	}
}

function subtasksProps() {
	return {
		subtasks: [],
		isLoading: false,
		error: null,
		canAddSubtask: true,
		onAddSubtask: vi.fn(),
		onOpenSubtask: vi.fn(),
	}
}

function commentsProps() {
	return {
		comments: [],
		isLoading: false,
		error: null,
		canLoadMore: false,
		canLoadOlder: false,
		draftBody: '',
		onDraftChange: vi.fn(),
		onSubmit: vi.fn(),
		isSubmitting: false,
		editingCommentId: null,
		editingBody: '',
		onStartEdit: vi.fn(),
		onEditBodyChange: vi.fn(),
		onConfirmEdit: vi.fn(),
		onCancelEdit: vi.fn(),
		onDelete: vi.fn(),
		onLoadOlder: vi.fn(),
		onLoadMore: vi.fn(),
	}
}

function baseProps() {
	return {
		open: true,
		mode: 'create' as const,
		title: 'Nouvelle tâche',
		isSaving: false,
		saveError: null as string | null,
		onSubmit: vi.fn(),
		onOpenChange: vi.fn(),
		fields: fieldsProps(),
	}
}

describe('TaskSheet — creation', () => {
	it('renders nothing when open is false', () => {
		render(<TaskSheet {...baseProps()} open={false} />)
		expect(screen.queryByRole('dialog')).toBeNull()
	})

	it('shows the Details tab only — no subtasks and no comments for a task not created yet', () => {
		render(<TaskSheet {...baseProps()} />)

		expect(screen.queryByRole('tab', { name: /Sous-tâches/ })).toBeNull()
		expect(screen.queryByRole('tab', { name: /Commentaires/ })).toBeNull()
	})

	it('calls onSubmit when create is clicked', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(<TaskSheet {...baseProps()} onSubmit={onSubmit} />)

		await user.click(screen.getByRole('button', { name: /Créer/ }))
		expect(onSubmit).toHaveBeenCalledTimes(1)
	})

	it('shows no delete button when creating', () => {
		render(<TaskSheet {...baseProps()} />)
		expect(screen.queryByRole('button', { name: /Supprimer/ })).toBeNull()
	})
})

describe('TaskSheet — editing', () => {
	function editProps() {
		return {
			...baseProps(),
			mode: 'edit' as const,
			title: 'Modifier la tâche',
			fields: { ...fieldsProps(), mode: 'edit' as const },
			onDelete: vi.fn(),
			subtasksTab: subtasksProps(),
			commentsTab: commentsProps(),
		}
	}

	it('shows the three tabs', () => {
		render(<TaskSheet {...editProps()} />)

		expect(screen.getByRole('tab', { name: /Détails/ })).toBeDefined()
		expect(screen.getByRole('tab', { name: /Sous-tâches/ })).toBeDefined()
		expect(screen.getByRole('tab', { name: /Commentaires/ })).toBeDefined()
	})

	it('switches to the subtasks tab on click', async () => {
		const user = userEvent.setup()
		render(<TaskSheet {...editProps()} />)

		await user.click(screen.getByRole('tab', { name: /Sous-tâches/ }))
		expect(
			screen.getByRole('button', { name: /Ajouter une sous-tâche/ }),
		).toBeDefined()
	})

	it('switches to the comments tab on click', async () => {
		const user = userEvent.setup()
		render(<TaskSheet {...editProps()} />)

		await user.click(screen.getByRole('tab', { name: /Commentaires/ }))
		expect(screen.getByLabelText('Nouveau commentaire')).toBeDefined()
	})

	it('shows the delete button when editing, even when the task has subtasks — deletion cascades server-side', () => {
		render(
			<TaskSheet
				{...editProps()}
				subtasksTab={{
					...subtasksProps(),
					subtasks: [
						{
							id: 'sub-1',
							title: 'Sous-tâche',
							status: 'PLANNED',
							assigneeCount: 0,
							inheritedWindow: false,
						},
					],
				}}
			/>,
		)
		expect(screen.getByRole('button', { name: /Supprimer/ })).toBeDefined()
	})

	it('hides the delete button when no onDelete is provided', () => {
		render(<TaskSheet {...editProps()} onDelete={undefined} />)
		expect(screen.queryByRole('button', { name: /Supprimer/ })).toBeNull()
	})

	it('calls onDelete when delete is clicked', async () => {
		const user = userEvent.setup()
		const onDelete = vi.fn()
		render(<TaskSheet {...editProps()} onDelete={onDelete} />)

		await user.click(screen.getByRole('button', { name: /Supprimer/ }))
		expect(onDelete).toHaveBeenCalledTimes(1)
	})
})

describe('TaskSheet — erreur de sauvegarde', () => {
	it('shows the error returned by the mutation', () => {
		render(<TaskSheet {...baseProps()} saveError="HTTP 409: conflit" />)
		expect(screen.getByText('HTTP 409: conflit')).toBeDefined()
	})
})

describe('TaskSheet — no network call', () => {
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

	it('fires no fetch on render', () => {
		render(<TaskSheet {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
