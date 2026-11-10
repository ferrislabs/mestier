import { useEffect, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCreateKnowledgeNote,
	useKnowledgeArticles,
} from '#/hooks/use-knowledge'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { KnowledgeDetailFeature } from '#/pages/knowledge/feature/knowledge-detail-feature'
import { useKnowledgeUploads } from '#/pages/knowledge/feature/use-knowledge-uploads'
import { filtersFromInputs, hasFilters } from '#/pages/knowledge/lib/filters'
import { KnowledgePage } from '#/pages/knowledge/ui/knowledge-page'
import {
	EMPTY_NOTE,
	NoteDialog,
	type NoteDraft,
} from '#/pages/knowledge/ui/note-dialog'
import { addTags, removeTag } from '#/pages/photo-library/lib/tags'

const SEARCH_DELAY_MS = 300

function useDebounced<T>(value: T, delay: number): T {
	const [debounced, setDebounced] = useState(value)
	useEffect(() => {
		const timer = setTimeout(() => setDebounced(value), delay)
		return () => clearTimeout(timer)
	}, [value, delay])
	return debounced
}

export function KnowledgeFeature() {
	const { activeOrganization } = useActiveOrganization()
	return (
		<KnowledgeWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

function KnowledgeWorkspace({ organizationId }: { organizationId: string }) {
	const canManage = useHasPermission('MANAGE_KNOWLEDGE')
	const [q, setQ] = useState('')
	const [tagInput, setTagInput] = useState('')
	const [page, setPage] = useState(1)
	const [openId, setOpenId] = useState<string | null>(null)
	const [note, setNote] = useState<NoteDraft | null>(null)

	const debouncedQ = useDebounced(q, SEARCH_DELAY_MS)
	const debouncedTags = useDebounced(tagInput, SEARCH_DELAY_MS)
	const filters = useMemo(
		() => filtersFromInputs({ q: debouncedQ, tagInput: debouncedTags }),
		[debouncedQ, debouncedTags],
	)

	const list = useKnowledgeArticles(organizationId, filters, page)
	const uploads = useKnowledgeUploads(organizationId)
	const createNote = useCreateKnowledgeNote(organizationId)

	const pagination = list.data?.pagination ?? null
	const submitNote = async () => {
		if (!note) return
		try {
			const created = await createNote.mutateAsync({
				title: note.title.trim(),
				body: note.body.trim(),
				tags: addTags(note.tags, note.tagInput),
			})
			setNote(null)
			setOpenId(created.id)
		} catch {}
	}

	return (
		<>
			<KnowledgePage
				articles={list.data?.articles ?? []}
				canManage={canManage}
				uploads={uploads.items}
				isUploading={uploads.isBusy}
				isLoading={list.isLoading}
				error={mutationErrorMessage(list.error)}
				hasFilters={hasFilters(filters)}
				page={page}
				hasNext={
					pagination?.next_page != null ||
					(pagination?.last_page != null && page < pagination.last_page)
				}
				total={pagination?.total ?? null}
				q={q}
				tagInput={tagInput}
				onFiles={(files) => {
					void uploads.enqueue(files)
				}}
				onClearUploads={uploads.clear}
				onNewNote={() => setNote(EMPTY_NOTE)}
				onQChange={(value) => {
					setQ(value)
					setPage(1)
				}}
				onTagInputChange={(value) => {
					setTagInput(value)
					setPage(1)
				}}
				onPageChange={setPage}
				onOpen={(article) => setOpenId(article.id)}
			/>
			{openId ? (
				<KnowledgeDetailFeature
					key={openId}
					organizationId={organizationId}
					articleId={openId}
					canManage={canManage}
					onClose={() => setOpenId(null)}
				/>
			) : null}
			<NoteDialog
				open={note !== null}
				draft={note ?? EMPTY_NOTE}
				isSaving={createNote.isPending}
				error={mutationErrorMessage(createNote.error)}
				onDraftChange={(change) =>
					setNote((current) => (current ? { ...current, ...change } : current))
				}
				onAddTags={() =>
					setNote((current) =>
						current
							? {
									...current,
									tags: addTags(current.tags, current.tagInput),
									tagInput: '',
								}
							: current,
					)
				}
				onRemoveTag={(tag) =>
					setNote((current) =>
						current
							? { ...current, tags: removeTag(current.tags, tag) }
							: current,
					)
				}
				onSubmit={() => {
					void submitNote()
				}}
				onOpenChange={(open) => {
					if (!open) setNote(null)
				}}
			/>
		</>
	)
}
