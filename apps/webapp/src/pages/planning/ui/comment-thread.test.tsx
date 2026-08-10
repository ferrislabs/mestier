import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { CommentThread } from '#/pages/planning/ui/comment-thread'

const COMMENTS = [
	{
		id: 'c1',
		task_id: 't1',
		organization_id: 'org1',
		author: { id: 'u1', display_name: 'Alice Dupont' },
		author_is_self: true,
		body: 'Premier message',
		created_at: '2026-08-10T08:00:00.000Z',
		updated_at: '2026-08-10T08:00:00.000Z',
	},
	{
		id: 'c2',
		task_id: 't1',
		organization_id: 'org1',
		author: { id: 'u2', display_name: 'Bob Martin' },
		author_is_self: false,
		body: 'Deuxième message',
		created_at: '2026-08-10T09:00:00.000Z',
		updated_at: '2026-08-10T09:00:00.000Z',
	},
]

function baseProps() {
	return {
		comments: COMMENTS,
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
		deletingCommentId: null,
		onLoadOlder: vi.fn(),
		onLoadMore: vi.fn(),
	}
}

describe('CommentThread — rendu', () => {
	it('renders every comment with its author and body', () => {
		render(<CommentThread {...baseProps()} />)

		expect(screen.getByText('Alice Dupont')).toBeDefined()
		expect(screen.getByText('Premier message')).toBeDefined()
		expect(screen.getByText('Bob Martin')).toBeDefined()
		expect(screen.getByText('Deuxième message')).toBeDefined()
	})

	it('renders comments in the order given — oldest first', () => {
		render(<CommentThread {...baseProps()} />)
		const bodies = screen
			.getAllByTestId('comment-body')
			.map((el) => el.textContent)
		expect(bodies).toEqual(['Premier message', 'Deuxième message'])
	})

	it('shows a loading state', () => {
		render(<CommentThread {...baseProps()} comments={[]} isLoading={true} />)
		expect(screen.getByText(/Chargement/)).toBeDefined()
	})

	it('shows an error state', () => {
		render(
			<CommentThread
				{...baseProps()}
				comments={[]}
				error="Échec du chargement"
			/>,
		)
		expect(screen.getByText('Échec du chargement')).toBeDefined()
	})

	it('shows an empty state when there are no comments', () => {
		render(<CommentThread {...baseProps()} comments={[]} />)
		expect(screen.getByText(/Aucun commentaire/)).toBeDefined()
	})
})

describe('CommentThread — actions reserved for the author', () => {
	it('shows the edit/delete actions only when author_is_self is true', () => {
		render(<CommentThread {...baseProps()} />)

		const ownComment = screen.getByTestId('comment-c1')
		const othersComment = screen.getByTestId('comment-c2')

		expect(ownComment.querySelector('[aria-label="Modifier"]')).not.toBeNull()
		expect(ownComment.querySelector('[aria-label="Supprimer"]')).not.toBeNull()
		expect(othersComment.querySelector('[aria-label="Modifier"]')).toBeNull()
		expect(othersComment.querySelector('[aria-label="Supprimer"]')).toBeNull()
	})

	it('never shows the actions on a comment whose author_is_self is false, whatever author it displays', () => {
		const sameDisplayNameButNotSelf = [
			{
				...COMMENTS[0],
				id: 'c3',
				author_is_self: false,
			},
		]
		render(
			<CommentThread {...baseProps()} comments={sameDisplayNameButNotSelf} />,
		)

		const comment = screen.getByTestId('comment-c3')
		expect(comment.querySelector('[aria-label="Modifier"]')).toBeNull()
		expect(comment.querySelector('[aria-label="Supprimer"]')).toBeNull()
	})

	it('starts editing a comment on click', async () => {
		const user = userEvent.setup()
		const onStartEdit = vi.fn()
		render(<CommentThread {...baseProps()} onStartEdit={onStartEdit} />)

		await user.click(
			screen
				.getByTestId('comment-c1')
				.querySelector('[aria-label="Modifier"]') as Element,
		)

		expect(onStartEdit).toHaveBeenCalledWith(COMMENTS[0])
	})

	it('renders the edit textarea for the comment currently being edited', () => {
		render(
			<CommentThread
				{...baseProps()}
				editingCommentId="c1"
				editingBody="Message corrigé"
			/>,
		)

		expect(screen.getByDisplayValue('Message corrigé')).toBeDefined()
	})

	it('calls onDelete with the comment when delete is confirmed', async () => {
		const user = userEvent.setup()
		const onDelete = vi.fn()
		render(<CommentThread {...baseProps()} onDelete={onDelete} />)

		await user.click(
			screen
				.getByTestId('comment-c1')
				.querySelector('[aria-label="Supprimer"]') as Element,
		)

		expect(onDelete).toHaveBeenCalledWith(COMMENTS[0])
	})
})

describe('CommentThread — pagination', () => {
	it('shows a "load older" control only when older comments exist', () => {
		const { rerender } = render(
			<CommentThread {...baseProps()} canLoadOlder={false} />,
		)
		expect(screen.queryByRole('button', { name: /plus anciens/i })).toBeNull()

		rerender(<CommentThread {...baseProps()} canLoadOlder={true} />)
		expect(screen.getByRole('button', { name: /plus anciens/i })).toBeDefined()
	})

	it('shows a "load more" control only when a next page exists', () => {
		const { rerender } = render(
			<CommentThread {...baseProps()} canLoadMore={false} />,
		)
		expect(screen.queryByRole('button', { name: /plus récents/i })).toBeNull()

		rerender(<CommentThread {...baseProps()} canLoadMore={true} />)
		expect(screen.getByRole('button', { name: /plus récents/i })).toBeDefined()
	})

	it('calls onLoadOlder/onLoadMore from their respective controls', async () => {
		const user = userEvent.setup()
		const onLoadOlder = vi.fn()
		const onLoadMore = vi.fn()
		render(
			<CommentThread
				{...baseProps()}
				canLoadOlder={true}
				canLoadMore={true}
				onLoadOlder={onLoadOlder}
				onLoadMore={onLoadMore}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /plus anciens/i }))
		await user.click(screen.getByRole('button', { name: /plus récents/i }))

		expect(onLoadOlder).toHaveBeenCalledTimes(1)
		expect(onLoadMore).toHaveBeenCalledTimes(1)
	})
})

describe('CommentThread — composition', () => {
	it('submits a new comment', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(
			<CommentThread
				{...baseProps()}
				draftBody="Un commentaire"
				onSubmit={onSubmit}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Envoyer' }))
		expect(onSubmit).toHaveBeenCalledTimes(1)
	})

	it('disables submit when the draft is blank', () => {
		render(<CommentThread {...baseProps()} draftBody="   " />)
		const submit = screen.getByRole('button', {
			name: 'Envoyer',
		}) as HTMLButtonElement
		expect(submit.disabled).toBe(true)
	})
})

describe('CommentThread — no network call', () => {
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
		render(<CommentThread {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
