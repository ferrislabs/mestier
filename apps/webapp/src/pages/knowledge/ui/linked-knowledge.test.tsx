import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { KnowledgeArticle } from '#/hooks/use-knowledge'
import {
	AttachKnowledgeDialog,
	LinkedKnowledge,
} from '#/pages/knowledge/ui/linked-knowledge'

const article: KnowledgeArticle = {
	id: 'a1',
	kind: 'file',
	title: 'Devis Dupont',
	body: null,
	file: { mime_type: 'application/pdf', size_bytes: 4096 },
	tags: [],
	created_at: '2026-11-10T08:00:00Z',
	updated_at: '2026-11-10T08:00:00Z',
}

describe('LinkedKnowledge', () => {
	it('lists the linked articles and opens one', async () => {
		const onOpen = vi.fn()
		render(
			<LinkedKnowledge
				articles={[article]}
				isLoading={false}
				error={null}
				canManage
				onOpen={onOpen}
				onUnlink={() => {}}
				onAttach={() => {}}
			/>,
		)
		expect(screen.getByText('Documents liés (1)')).toBeTruthy()
		await userEvent.click(screen.getByText('Devis Dupont'))
		expect(onOpen).toHaveBeenCalledWith(article)
	})

	it('detaches and attaches only for someone who can manage', async () => {
		const onUnlink = vi.fn()
		const onAttach = vi.fn()
		const { rerender } = render(
			<LinkedKnowledge
				articles={[article]}
				isLoading={false}
				error={null}
				canManage
				onOpen={() => {}}
				onUnlink={onUnlink}
				onAttach={onAttach}
			/>,
		)
		await userEvent.click(
			screen.getByRole('button', { name: 'Détacher Devis Dupont' }),
		)
		expect(onUnlink).toHaveBeenCalledWith(article)
		await userEvent.click(
			screen.getByRole('button', { name: /Rattacher un document/ }),
		)
		expect(onAttach).toHaveBeenCalled()

		rerender(
			<LinkedKnowledge
				articles={[article]}
				isLoading={false}
				error={null}
				canManage={false}
				onOpen={() => {}}
				onUnlink={onUnlink}
				onAttach={onAttach}
			/>,
		)
		expect(screen.queryByRole('button', { name: /Détacher/ })).toBeNull()
		expect(screen.queryByRole('button', { name: /Rattacher/ })).toBeNull()
	})

	it('says when nothing is attached', () => {
		render(
			<LinkedKnowledge
				articles={[]}
				isLoading={false}
				error={null}
				canManage={false}
				onOpen={() => {}}
				onUnlink={() => {}}
				onAttach={() => {}}
			/>,
		)
		expect(screen.getByText('Aucun document ni note rattaché.')).toBeTruthy()
	})
})

describe('AttachKnowledgeDialog', () => {
	it('attaches the article that is picked', async () => {
		const onPick = vi.fn()
		render(
			<AttachKnowledgeDialog
				open
				query=""
				candidates={[article]}
				isLoading={false}
				isAttaching={false}
				error={null}
				onQueryChange={() => {}}
				onPick={onPick}
				onOpenChange={() => {}}
			/>,
		)
		await userEvent.click(screen.getByText('Devis Dupont'))
		expect(onPick).toHaveBeenCalledWith(article)
	})
})
