import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationRuns,
	useAutomationWorkflows,
	useCreateFolder,
	useDeleteFolder,
	useDeleteWorkflow,
	useUpdateFolder,
	useUpdateWorkflow,
	type Workflow,
} from '#/hooks/use-automation'
import { useWorkflowActions } from '#/pages/automation/feature/use-workflow-actions'
import { useWorkflowFolders } from '#/pages/automation/feature/use-workflow-folders'
import {
	canDropOn,
	childFolders,
	countWorkflowsInSubtree,
	type DragItem,
	findFolder,
	folderPath,
	moveTargets,
	type WorkflowFolder,
} from '#/pages/automation/lib/folders'
import {
	ALL_TAGS,
	collectTags,
	filterWorkflows,
	sortWorkflows,
	type WorkflowSortKey,
} from '#/pages/automation/lib/workflow-list'
import { latestRunByWorkflow } from '#/pages/automation/lib/workflow-runs'
import {
	EMPTY_WORKFLOW_FORM,
	type WorkflowFormValues,
} from '#/pages/automation/types'
import {
	AutomationWorkflowsList,
	type WorkflowRow,
} from '#/pages/automation/ui/automation-workflows-list'
import {
	FolderMoveDialog,
	ROOT_TARGET,
} from '#/pages/automation/ui/folder-move-dialog'
import { FolderNameDialog } from '#/pages/automation/ui/folder-name-dialog'
import { WorkflowEditorDialog } from '#/pages/automation/ui/workflow-editor-dialog'

export function AutomationWorkflowsFeature({
	folderId,
}: {
	folderId?: string
}) {
	const { activeOrganization } = useActiveOrganization()

	return (
		<AutomationWorkflowsWorkspace
			key={`${activeOrganization.id}:${folderId ?? ''}`}
			folderId={folderId}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
		/>
	)
}

interface Draft {
	id: string
	values: WorkflowFormValues
}

type FolderNameDraft =
	| { kind: 'create'; parentId: string | null; name: string }
	| { kind: 'rename'; id: string; name: string }

type MoveDraft =
	| { kind: 'folder'; id: string; label: string; target: string }
	| { kind: 'workflow'; id: string; label: string; target: string }

function messageOf(caught: unknown, fallback: string): string {
	return caught instanceof Error && caught.message ? caught.message : fallback
}

function AutomationWorkflowsWorkspace({
	organizationId,
	organizationSlug,
	folderId,
}: {
	organizationId: string
	organizationSlug: string
	folderId?: string
}) {
	const [draft, setDraft] = useState<Draft | null>(null)
	const [nameDraft, setNameDraft] = useState<FolderNameDraft | null>(null)
	const [moveDraft, setMoveDraft] = useState<MoveDraft | null>(null)
	const [dragItem, setDragItem] = useState<DragItem | null>(null)
	const [dialogError, setDialogError] = useState<string | null>(null)
	const [query, setQuery] = useState('')
	const [tag, setTag] = useState(ALL_TAGS)
	const [sort, setSort] = useState<WorkflowSortKey>('updated')

	const workflows = useAutomationWorkflows(organizationId)
	const runs = useAutomationRuns(organizationId)
	const updateWorkflow = useUpdateWorkflow()
	const deleteWorkflow = useDeleteWorkflow()
	const createFolder = useCreateFolder()
	const updateFolder = useUpdateFolder()
	const deleteFolder = useDeleteFolder()

	const folders = useWorkflowFolders(organizationId)
	const workflowActions = useWorkflowActions(organizationId)
	const [actionError, setActionError] = useState<string | null>(null)

	const items: Workflow[] = workflows.data?.data ?? []
	const latestRuns = latestRunByWorkflow(runs.data?.data ?? [])

	const folder = folderId ? findFolder(folders.data, folderId) : undefined

	const searching = query.trim() !== ''
	const pathLabel = (id: string | null) =>
		id
			? folderPath(folders.data, id)
					.map((item) => item.name)
					.join(' › ')
			: null

	const rows: WorkflowRow[] = items.map((workflow) => ({
		id: workflow.id,
		name: workflow.name,
		description: workflow.description ?? null,
		enabled: workflow.enabled,
		lastRun: latestRuns.get(workflow.id) ?? null,
		updatedAt: workflow.updated_at,
		tags: workflow.tags,
		folderId: workflow.folder_id ?? null,
		folderLabel:
			searching && workflow.folder_id ? pathLabel(workflow.folder_id) : null,
	}))

	const inFolder = rows.filter((row) => row.folderId === (folderId ?? null))
	const scoped = searching ? rows : inFolder
	const tags = collectTags(scoped)
	const visible = sortWorkflows(filterWorkflows(scoped, query, tag), sort)
	const directFolders = childFolders(folders.data, folderId ?? null)
	const subfolders = (searching ? [] : directFolders).map((subfolder) => ({
		folder: subfolder,
		workflowCount: countWorkflowsInSubtree(folders.data, rows, subfolder.id),
	}))

	const error =
		workflows.error ??
		folders.error ??
		runs.error ??
		updateWorkflow.error ??
		deleteWorkflow.error
	const errorMessage = error?.message ?? actionError

	const sourceOf = (row: WorkflowRow) => ({
		id: row.id,
		name: row.name,
		description: row.description,
		folderId: row.folderId,
		tags: row.tags,
	})

	const duplicate = async (row: WorkflowRow) => {
		setActionError(null)
		try {
			await workflowActions.duplicate(sourceOf(row))
		} catch (caught) {
			setActionError(
				caught instanceof Error && caught.message
					? caught.message
					: 'La duplication a échoué.',
			)
		}
	}

	const exportRow = async (row: WorkflowRow) => {
		setActionError(null)
		try {
			await workflowActions.exportWorkflow(sourceOf(row))
		} catch (caught) {
			setActionError(
				caught instanceof Error && caught.message
					? caught.message
					: 'L’export a échoué.',
			)
		}
	}

	const closeNameDialog = () => {
		setNameDraft(null)
		setDialogError(null)
	}

	const closeMoveDialog = () => {
		setMoveDraft(null)
		setDialogError(null)
	}

	const submitFolderName = async () => {
		if (!nameDraft) return
		setDialogError(null)
		const name = nameDraft.name.trim()
		try {
			if (nameDraft.kind === 'create') {
				await createFolder.mutateAsync({
					path: { organization_id: organizationId },
					body: { name, parent_id: nameDraft.parentId },
				})
			} else {
				await updateFolder.mutateAsync({
					path: { organization_id: organizationId, folder_id: nameDraft.id },
					body: { name },
				})
			}
			closeNameDialog()
		} catch (caught) {
			setDialogError(messageOf(caught, 'L’enregistrement a échoué.'))
		}
	}

	const moveItem = (
		kind: DragItem['kind'],
		id: string,
		target: string | null,
	) =>
		kind === 'folder'
			? updateFolder.mutateAsync({
					path: { organization_id: organizationId, folder_id: id },
					body: { parent_id: target },
				})
			: updateWorkflow.mutateAsync({
					path: { organization_id: organizationId, workflow_id: id },
					body: { folder_id: target },
				})

	const submitMove = async () => {
		if (!moveDraft) return
		setDialogError(null)
		const target = moveDraft.target === ROOT_TARGET ? null : moveDraft.target
		try {
			await moveItem(moveDraft.kind, moveDraft.id, target)
			closeMoveDialog()
		} catch (caught) {
			setDialogError(messageOf(caught, 'Le déplacement a échoué.'))
		}
	}

	const dropOn = async (target: string | null) => {
		const item = dragItem
		setDragItem(null)
		if (!item || !canDropOn(folders.data, item, target)) return
		setActionError(null)
		try {
			await moveItem(item.kind, item.id, target)
		} catch (caught) {
			setActionError(messageOf(caught, 'Le déplacement a échoué.'))
		}
	}

	const removeFolder = async (target: WorkflowFolder) => {
		setActionError(null)
		try {
			await deleteFolder.mutateAsync({
				path: { organization_id: organizationId, folder_id: target.id },
			})
		} catch (caught) {
			setActionError(
				messageOf(
					caught,
					'Ce dossier n’a pas pu être supprimé. Vérifiez qu’il est vide.',
				),
			)
		}
	}

	const submitDraft = async () => {
		if (!draft) return

		await updateWorkflow.mutateAsync({
			path: { organization_id: organizationId, workflow_id: draft.id },
			body: {
				name: draft.values.name.trim(),
				description: draft.values.description.trim() || null,
			},
		})

		setDraft(null)
	}

	return (
		<>
			<AutomationWorkflowsList
				organizationSlug={organizationSlug}
				workflows={visible}
				isLoading={workflows.isLoading}
				error={errorMessage}
				folder={folder ?? null}
				folderPath={folderId ? folderPath(folders.data, folderId) : []}
				folderMissing={
					folderId !== undefined &&
					!folders.isLoading &&
					!folders.error &&
					folder === undefined
				}
				folderPending={folderId !== undefined && folders.isLoading}
				subfolders={subfolders}
				hasWorkflowsInScope={scoped.length > 0 || directFolders.length > 0}
				query={query}
				onQueryChange={setQuery}
				tag={tag}
				tags={tags}
				onTagChange={setTag}
				sort={sort}
				onSortChange={setSort}
				onResetFilters={() => {
					setQuery('')
					setTag(ALL_TAGS)
				}}
				dragItem={dragItem}
				canDropOn={(target) =>
					dragItem !== null && canDropOn(folders.data, dragItem, target)
				}
				onDragStart={setDragItem}
				onDragEnd={() => setDragItem(null)}
				onDropOn={(target) => void dropOn(target)}
				onCreateFolder={() =>
					setNameDraft({ kind: 'create', parentId: folderId ?? null, name: '' })
				}
				onRenameFolder={(target) =>
					setNameDraft({ kind: 'rename', id: target.id, name: target.name })
				}
				onMoveFolder={(target) =>
					setMoveDraft({
						kind: 'folder',
						id: target.id,
						label: target.name,
						target: target.parentId ?? ROOT_TARGET,
					})
				}
				onDeleteFolder={(target) => void removeFolder(target)}
				onMove={(row) =>
					setMoveDraft({
						kind: 'workflow',
						id: row.id,
						label: row.name,
						target: row.folderId ?? ROOT_TARGET,
					})
				}
				onRename={(row) =>
					setDraft({
						id: row.id,
						values: { name: row.name, description: row.description ?? '' },
					})
				}
				onToggleEnabled={(row) =>
					void updateWorkflow.mutateAsync({
						path: { organization_id: organizationId, workflow_id: row.id },
						body: { enabled: !row.enabled },
					})
				}
				onDuplicate={(row) => void duplicate(row)}
				onExport={(row) => void exportRow(row)}
				onArchive={(row) =>
					updateWorkflow.mutate({
						path: { organization_id: organizationId, workflow_id: row.id },
						body: { archived: true },
					})
				}
				onDelete={(row) =>
					void deleteWorkflow.mutateAsync({
						path: { organization_id: organizationId, workflow_id: row.id },
					})
				}
			/>

			<FolderNameDialog
				open={nameDraft !== null}
				title={
					nameDraft?.kind === 'rename'
						? 'Renommer le dossier'
						: 'Nouveau dossier'
				}
				submitLabel={nameDraft?.kind === 'rename' ? 'Enregistrer' : 'Créer'}
				name={nameDraft?.name ?? ''}
				isPending={createFolder.isPending || updateFolder.isPending}
				error={dialogError}
				onNameChange={(name) =>
					setNameDraft((current) => (current ? { ...current, name } : current))
				}
				onOpenChange={(open) => {
					if (!open) closeNameDialog()
				}}
				onSubmit={() => void submitFolderName()}
			/>

			<FolderMoveDialog
				open={moveDraft !== null}
				title={moveDraft ? `Déplacer « ${moveDraft.label} »` : ''}
				targets={moveTargets(
					folders.data,
					moveDraft?.kind === 'folder' ? moveDraft.id : undefined,
				)}
				value={moveDraft?.target ?? ROOT_TARGET}
				isPending={updateFolder.isPending || updateWorkflow.isPending}
				error={dialogError}
				onValueChange={(target) =>
					setMoveDraft((current) =>
						current ? { ...current, target } : current,
					)
				}
				onOpenChange={(open) => {
					if (!open) closeMoveDialog()
				}}
				onSubmit={() => void submitMove()}
			/>

			<WorkflowEditorDialog
				open={draft !== null}
				editingName={
					draft ? (rows.find((row) => row.id === draft.id)?.name ?? '') : ''
				}
				values={draft?.values ?? EMPTY_WORKFLOW_FORM}
				isPending={updateWorkflow.isPending}
				error={error?.message ?? null}
				onOpenChange={(open) => {
					if (!open) setDraft(null)
				}}
				onValuesChange={(patch) =>
					setDraft((current) =>
						current
							? { ...current, values: { ...current.values, ...patch } }
							: current,
					)
				}
				onSubmit={() => void submitDraft()}
			/>
		</>
	)
}
