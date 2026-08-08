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

describe('TaskSheet — création', () => {
	it("n'affiche rien quand open est faux", () => {
		render(<TaskSheet {...baseProps()} open={false} />)
		expect(screen.queryByRole('dialog')).toBeNull()
	})

	it('affiche uniquement l’onglet Détails — pas de sous-tâches ni de commentaires pour une tâche pas encore créée', () => {
		render(<TaskSheet {...baseProps()} />)

		expect(screen.queryByRole('tab', { name: /Sous-tâches/ })).toBeNull()
		expect(screen.queryByRole('tab', { name: /Commentaires/ })).toBeNull()
	})

	it('appelle onSubmit au clic sur créer', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(<TaskSheet {...baseProps()} onSubmit={onSubmit} />)

		await user.click(screen.getByRole('button', { name: /Créer/ }))
		expect(onSubmit).toHaveBeenCalledTimes(1)
	})

	it("n'affiche pas de bouton supprimer en création", () => {
		render(<TaskSheet {...baseProps()} />)
		expect(screen.queryByRole('button', { name: /Supprimer/ })).toBeNull()
	})
})

describe('TaskSheet — édition', () => {
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

	it('affiche les trois onglets', () => {
		render(<TaskSheet {...editProps()} />)

		expect(screen.getByRole('tab', { name: /Détails/ })).toBeDefined()
		expect(screen.getByRole('tab', { name: /Sous-tâches/ })).toBeDefined()
		expect(screen.getByRole('tab', { name: /Commentaires/ })).toBeDefined()
	})

	it('bascule vers l’onglet sous-tâches au clic', async () => {
		const user = userEvent.setup()
		render(<TaskSheet {...editProps()} />)

		await user.click(screen.getByRole('tab', { name: /Sous-tâches/ }))
		expect(
			screen.getByRole('button', { name: /Ajouter une sous-tâche/ }),
		).toBeDefined()
	})

	it('bascule vers l’onglet commentaires au clic', async () => {
		const user = userEvent.setup()
		render(<TaskSheet {...editProps()} />)

		await user.click(screen.getByRole('tab', { name: /Commentaires/ }))
		expect(screen.getByLabelText('Nouveau commentaire')).toBeDefined()
	})

	it('affiche le bouton supprimer en édition, même quand la tâche a des sous-tâches — la suppression cascade côté serveur', () => {
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

	it("masque le bouton supprimer quand aucun onDelete n'est fourni", () => {
		render(<TaskSheet {...editProps()} onDelete={undefined} />)
		expect(screen.queryByRole('button', { name: /Supprimer/ })).toBeNull()
	})

	it('appelle onDelete au clic sur supprimer', async () => {
		const user = userEvent.setup()
		const onDelete = vi.fn()
		render(<TaskSheet {...editProps()} onDelete={onDelete} />)

		await user.click(screen.getByRole('button', { name: /Supprimer/ }))
		expect(onDelete).toHaveBeenCalledTimes(1)
	})
})

describe('TaskSheet — erreur de sauvegarde', () => {
	it('affiche l’erreur renvoyée par la mutation', () => {
		render(<TaskSheet {...baseProps()} saveError="HTTP 409: conflit" />)
		expect(screen.getByText('HTTP 409: conflit')).toBeDefined()
	})
})

describe('TaskSheet — pas d’appel réseau', () => {
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
		render(<TaskSheet {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
