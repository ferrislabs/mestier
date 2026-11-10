import { useEffect, useMemo, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	type KnowledgeFolder,
	useCreateKnowledgeFolder,
	useCreateKnowledgeNote,
	useDeleteKnowledgeFolder,
	useKnowledgeArticles,
	useKnowledgeFolders,
	useMoveKnowledgeArticles,
	useUpdateKnowledgeFolder,
} from '#/hooks/use-knowledge'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { KnowledgeDetailFeature } from '#/pages/knowledge/feature/knowledge-detail-feature'
import { useKnowledgeUploads } from '#/pages/knowledge/feature/use-knowledge-uploads'
import { filtersFromInputs, hasFilters } from '#/pages/knowledge/lib/filters'
import {
	canCreateIn,
	canMoveFolderTo,
	childrenByParent,
	folderNameError,
	pathTo,
} from '#/pages/knowledge/lib/folders'
import { FolderNameDialog } from '#/pages/knowledge/ui/folder-name-dialog'
import { FolderPickerDialog } from '#/pages/knowledge/ui/folder-picker-dialog'
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

interface KnowledgeFeatureProps {
	/** The open folder, from the address. `null` is the root. */
	folderId: string | null
	onFolderChange: (folderId: string | null) => void
}

export function KnowledgeFeature(props: KnowledgeFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	return (
		<KnowledgeWorkspace
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			{...props}
		/>
	)
}

/** A folder being named: a new one under `parentId`, or `folder` renamed. */
type NameDraft =
	| { mode: 'create'; parentId: string | null; name: string }
	| { mode: 'rename'; folder: KnowledgeFolder; name: string }

/** What the destination picker moves. */
type MoveDraft =
	| { mode: 'articles'; articleIds: string[]; target: string | null }
	| { mode: 'folder'; folder: KnowledgeFolder; target: string | null }

function KnowledgeWorkspace({
	organizationId,
	folderId,
	onFolderChange,
}: KnowledgeFeatureProps & { organizationId: string }) {
	const canManage = useHasPermission('MANAGE_KNOWLEDGE')
	const [q, setQ] = useState('')
	const [tagInput, setTagInput] = useState('')
	const [page, setPage] = useState(1)
	const [openId, setOpenId] = useState<string | null>(null)
	const [note, setNote] = useState<NoteDraft | null>(null)
	const [expanded, setExpanded] = useState<Set<string>>(new Set())
	const [selected, setSelected] = useState<Set<string>>(new Set())
	const [nameDraft, setNameDraft] = useState<NameDraft | null>(null)
	const [moveDraft, setMoveDraft] = useState<MoveDraft | null>(null)
	const [toDelete, setToDelete] = useState<KnowledgeFolder | null>(null)
	const [dropError, setDropError] = useState<string | null>(null)

	const debouncedQ = useDebounced(q, SEARCH_DELAY_MS)
	const debouncedTags = useDebounced(tagInput, SEARCH_DELAY_MS)
	const filters = useMemo(
		() => filtersFromInputs({ q: debouncedQ, tagInput: debouncedTags }),
		[debouncedQ, debouncedTags],
	)
	const searching = hasFilters(filters)

	const folderList = useKnowledgeFolders(organizationId)
	const folders = useMemo(() => folderList.data ?? [], [folderList.data])
	const known = folderId === null || folders.some((f) => f.id === folderId)
	const currentFolderId = folderList.isSuccess && !known ? null : folderId
	const path = useMemo(
		() => pathTo(folders, currentFolderId),
		[folders, currentFolderId],
	)
	const subfolders = useMemo(
		() => childrenByParent(folders).get(currentFolderId) ?? [],
		[folders, currentFolderId],
	)

	const scope = useMemo(
		() => ({ folderId: currentFolderId, recursive: searching }),
		[currentFolderId, searching],
	)
	const list = useKnowledgeArticles(organizationId, filters, page, scope)
	const uploads = useKnowledgeUploads(organizationId, currentFolderId)
	const createNote = useCreateKnowledgeNote(organizationId)
	const createFolder = useCreateKnowledgeFolder(organizationId)
	const updateFolder = useUpdateKnowledgeFolder(organizationId)
	const deleteFolder = useDeleteKnowledgeFolder(organizationId)
	const moveArticles = useMoveKnowledgeArticles(organizationId)

	// A folder that no longer exists (deleted elsewhere) sends back to the root.
	useEffect(() => {
		if (folderList.isSuccess && !known) onFolderChange(null)
	}, [folderList.isSuccess, known, onFolderChange])

	// Opening a folder unfolds the tree down to it.
	useEffect(() => {
		if (path.length === 0) return
		setExpanded((current) => {
			const missing = path.filter((folder) => !current.has(folder.id))
			if (missing.length === 0) return current
			return new Set([...current, ...missing.map((folder) => folder.id)])
		})
	}, [path])

	const openFolder = (id: string | null) => {
		setPage(1)
		setSelected(new Set())
		setDropError(null)
		onFolderChange(id)
	}

	const pagination = list.data?.pagination ?? null

	const submitNote = async () => {
		if (!note) return
		try {
			const created = await createNote.mutateAsync({
				title: note.title.trim(),
				body: note.body.trim(),
				tags: addTags(note.tags, note.tagInput),
				folderId: currentFolderId,
			})
			setNote(null)
			setOpenId(created.id)
		} catch {}
	}

	const nameParent =
		nameDraft === null
			? null
			: nameDraft.mode === 'create'
				? nameDraft.parentId
				: nameDraft.folder.parent_id
	const nameCheck =
		nameDraft === null || nameDraft.name.trim() === ''
			? null
			: folderNameError(
					folders,
					nameParent,
					nameDraft.name,
					nameDraft.mode === 'rename' ? nameDraft.folder.id : null,
				)
	const nameMutation =
		nameDraft?.mode === 'rename' ? updateFolder : createFolder

	const submitName = async () => {
		if (!nameDraft || nameCheck) return
		try {
			if (nameDraft.mode === 'create') {
				await createFolder.mutateAsync({
					name: nameDraft.name.trim(),
					parentId: nameDraft.parentId,
				})
			} else {
				await updateFolder.mutateAsync({
					id: nameDraft.folder.id,
					name: nameDraft.name.trim(),
				})
			}
			setNameDraft(null)
		} catch {}
	}

	const moveMutation =
		moveDraft?.mode === 'folder' ? updateFolder : moveArticles
	const isAllowedTarget = (target: string | null) => {
		if (!moveDraft) return false
		if (moveDraft.mode === 'folder') {
			return (
				target !== moveDraft.folder.parent_id &&
				canMoveFolderTo(folders, moveDraft.folder.id, target)
			)
		}
		return true
	}

	const submitMove = async () => {
		if (!moveDraft || !isAllowedTarget(moveDraft.target)) return
		try {
			if (moveDraft.mode === 'folder') {
				await updateFolder.mutateAsync({
					id: moveDraft.folder.id,
					parentId: moveDraft.target,
				})
			} else {
				await moveArticles.mutateAsync({
					articleIds: moveDraft.articleIds,
					folderId: moveDraft.target,
				})
				setSelected(new Set())
			}
			setMoveDraft(null)
		} catch {}
	}

	const dropArticles = async (target: string | null, articleIds: string[]) => {
		setDropError(null)
		try {
			await moveArticles.mutateAsync({ articleIds, folderId: target })
			setSelected(new Set())
		} catch (error) {
			setDropError(mutationErrorMessage(error))
		}
	}

	const confirmDelete = async () => {
		if (!toDelete) return
		try {
			await deleteFolder.mutateAsync(toDelete.id)
			if (path.some((folder) => folder.id === toDelete.id)) {
				openFolder(toDelete.parent_id)
			}
			setToDelete(null)
		} catch {}
	}

	return (
		<>
			<KnowledgePage
				folders={folders}
				currentFolderId={currentFolderId}
				path={path}
				subfolders={subfolders}
				expanded={expanded}
				articles={list.data?.articles ?? []}
				selected={selected}
				canManage={canManage}
				canCreateFolder={canCreateIn(folders, currentFolderId)}
				uploads={uploads.items}
				isUploading={uploads.isBusy}
				isLoading={list.isLoading || folderList.isLoading}
				error={
					dropError ??
					mutationErrorMessage(list.error) ??
					mutationErrorMessage(folderList.error)
				}
				hasFilters={searching}
				page={page}
				hasNext={
					pagination?.next_page != null ||
					(pagination?.last_page != null && page < pagination.last_page)
				}
				total={pagination?.total ?? null}
				q={q}
				tagInput={tagInput}
				onOpenFolder={openFolder}
				onToggleFolder={(id) =>
					setExpanded((current) => {
						const next = new Set(current)
						if (next.has(id)) next.delete(id)
						else next.add(id)
						return next
					})
				}
				onNewFolder={() =>
					setNameDraft({ mode: 'create', parentId: currentFolderId, name: '' })
				}
				onRenameFolder={(folder) =>
					setNameDraft({ mode: 'rename', folder, name: folder.name })
				}
				onMoveFolder={(folder) =>
					setMoveDraft({ mode: 'folder', folder, target: folder.parent_id })
				}
				onDeleteFolder={setToDelete}
				onDropArticles={(target, ids) => {
					void dropArticles(target, ids)
				}}
				onToggleSelected={(id) =>
					setSelected((current) => {
						const next = new Set(current)
						if (next.has(id)) next.delete(id)
						else next.add(id)
						return next
					})
				}
				onClearSelection={() => setSelected(new Set())}
				onMoveSelected={() =>
					setMoveDraft({
						mode: 'articles',
						articleIds: [...selected],
						target: currentFolderId,
					})
				}
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
			<FolderNameDialog
				open={nameDraft !== null}
				title={
					nameDraft?.mode === 'rename'
						? 'Renommer le dossier'
						: 'Nouveau dossier'
				}
				description={
					nameDraft?.mode === 'rename'
						? 'Le dossier garde son contenu.'
						: `Le dossier est créé dans « ${path.at(-1)?.name ?? 'Drive'} ».`
				}
				submitLabel={nameDraft?.mode === 'rename' ? 'Renommer' : 'Créer'}
				name={nameDraft?.name ?? ''}
				error={nameCheck ?? mutationErrorMessage(nameMutation.error)}
				isSaving={nameMutation.isPending}
				onNameChange={(name) =>
					setNameDraft((current) => (current ? { ...current, name } : current))
				}
				onSubmit={() => {
					void submitName()
				}}
				onOpenChange={(open) => {
					if (!open) {
						setNameDraft(null)
						createFolder.reset()
						updateFolder.reset()
					}
				}}
			/>
			<FolderPickerDialog
				open={moveDraft !== null}
				title={
					moveDraft?.mode === 'folder'
						? `Déplacer « ${moveDraft.folder.name} »`
						: moveDraft && moveDraft.articleIds.length > 1
							? `Déplacer ${moveDraft.articleIds.length} éléments`
							: 'Déplacer l’élément'
				}
				description="Choisis le dossier de destination."
				folders={folders}
				selected={moveDraft?.target ?? null}
				isAllowed={isAllowedTarget}
				error={mutationErrorMessage(moveMutation.error)}
				isSaving={moveMutation.isPending}
				onSelect={(target) =>
					setMoveDraft((current) =>
						current ? { ...current, target } : current,
					)
				}
				onConfirm={() => {
					void submitMove()
				}}
				onOpenChange={(open) => {
					if (!open) {
						setMoveDraft(null)
						moveArticles.reset()
						updateFolder.reset()
					}
				}}
			/>
			<ConfirmDialog
				open={toDelete !== null}
				onOpenChange={(open) => {
					if (!open) {
						setToDelete(null)
						deleteFolder.reset()
					}
				}}
				title={`Supprimer « ${toDelete?.name ?? ''} » ?`}
				description="Seul un dossier vide peut être supprimé. Son contenu n’est jamais effacé avec lui."
				confirmLabel="Supprimer"
				destructive
				disabled={deleteFolder.isPending}
				onConfirm={() => {
					void confirmDelete()
				}}
			>
				{deleteFolder.error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{mutationErrorMessage(deleteFolder.error)}
					</p>
				) : null}
			</ConfirmDialog>
		</>
	)
}
