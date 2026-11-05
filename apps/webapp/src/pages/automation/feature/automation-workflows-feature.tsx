import { useNavigate, useSearch } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationRuns,
	useAutomationWorkflows,
	useCreateWorkflow,
	useDeleteWorkflow,
	useUpdateWorkflow,
	type Workflow,
} from '#/hooks/use-automation'
import { useWorkflowActions } from '#/pages/automation/feature/use-workflow-actions'
import {
	useArchivedWorkflowIds,
	useWorkflowFolders,
	useWorkflowPlacementActions,
	useWorkflowPlacements,
} from '#/pages/automation/feature/use-workflow-folders'
import {
	childFolders,
	countWorkflowsInSubtree,
	findFolder,
	folderPath,
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
	id: string | null
	values: WorkflowFormValues
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
	const navigate = useNavigate()
	const wantsNew = useSearch({
		strict: false,
		select: (search) => (search as { nouveau?: unknown }).nouveau,
	})
	// The side nav's "Nouveau workflow" lands here with `?nouveau=1`: open the
	// create dialog once, then drop the param so a reload does not reopen it.
	useEffect(() => {
		if (!wantsNew) return
		setDraft({ id: null, values: EMPTY_WORKFLOW_FORM })
		void navigate({
			to: '.',
			search: (current: Record<string, unknown>) => ({
				...current,
				nouveau: undefined,
			}),
			replace: true,
		})
	}, [wantsNew, navigate])
	const [query, setQuery] = useState('')
	const [tag, setTag] = useState(ALL_TAGS)
	const [sort, setSort] = useState<WorkflowSortKey>('updated')

	const workflows = useAutomationWorkflows(organizationId)
	const runs = useAutomationRuns(organizationId)
	const createWorkflow = useCreateWorkflow(organizationId)
	const updateWorkflow = useUpdateWorkflow()
	const deleteWorkflow = useDeleteWorkflow()

	const folders = useWorkflowFolders()
	const placements = useWorkflowPlacements()
	const archived = useArchivedWorkflowIds()
	const placementActions = useWorkflowPlacementActions()
	const workflowActions = useWorkflowActions(organizationId)
	const [actionError, setActionError] = useState<string | null>(null)

	const allItems: Workflow[] = workflows.data?.data ?? []
	const items = allItems.filter((workflow) => !archived.has(workflow.id))
	const { rememberNames } = placementActions

	useEffect(() => {
		rememberNames(allItems)
	}, [allItems, rememberNames])
	const latestRuns = latestRunByWorkflow(runs.data?.data ?? [])

	const folder = folderId ? findFolder(folders.data, folderId) : undefined
	const pathLabel = (id: string | null) =>
		id
			? folderPath(folders.data, id)
					.map((item) => item.name)
					.join(' › ')
			: null

	const rows: WorkflowRow[] = items.map((workflow) => {
		const placement = placements.data[workflow.id]
		return {
			id: workflow.id,
			name: workflow.name,
			description: workflow.description ?? null,
			enabled: workflow.enabled,
			lastRun: latestRuns.get(workflow.id) ?? null,
			updatedAt: workflow.updated_at,
			tags: placement?.tags ?? [],
			folderLabel: folder ? null : pathLabel(placement?.folderId ?? null),
		}
	})

	const folderIdOf = (id: string) => placements.data[id]?.folderId ?? null
	const scoped = folderId
		? rows.filter((row) => folderIdOf(row.id) === folderId)
		: rows
	const tags = useMemo(() => collectTags(scoped), [scoped])
	const visible = sortWorkflows(filterWorkflows(scoped, query, tag), sort)
	const subfolders = (folderId ? childFolders(folders.data, folderId) : []).map(
		(subfolder) => ({
			folder: subfolder,
			workflowCount: countWorkflowsInSubtree(
				folders.data,
				rows.map((row) => ({ folderId: folderIdOf(row.id) })),
				subfolder.id,
			),
		}),
	)

	const isPending = createWorkflow.isPending || updateWorkflow.isPending

	const error =
		workflows.error ??
		runs.error ??
		createWorkflow.error ??
		updateWorkflow.error ??
		deleteWorkflow.error
	const errorMessage = error?.message ?? actionError

	const sourceOf = (row: WorkflowRow) => ({
		id: row.id,
		name: row.name,
		description: row.description,
		tags: row.tags,
	})

	const duplicate = async (row: WorkflowRow) => {
		setActionError(null)
		try {
			const copyId = await workflowActions.duplicate(sourceOf(row))
			placementActions.place(copyId, {
				folderId: folderIdOf(row.id),
				tags: row.tags,
			})
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

	const submitDraft = async () => {
		if (!draft) return

		if (draft.id === null) {
			await createWorkflow.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: draft.values.name.trim(),
					description: draft.values.description.trim() || null,
				},
			})
		} else {
			await updateWorkflow.mutateAsync({
				path: { organization_id: organizationId, workflow_id: draft.id },
				body: {
					name: draft.values.name.trim(),
					description: draft.values.description.trim() || null,
				},
			})
		}

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
				folderMissing={folderId !== undefined && folder === undefined}
				subfolders={subfolders}
				hasWorkflowsInScope={scoped.length > 0}
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
				onCreate={() => setDraft({ id: null, values: EMPTY_WORKFLOW_FORM })}
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
				onArchive={(row) => placementActions.archive(row.id)}
				onDelete={(row) =>
					void deleteWorkflow.mutateAsync({
						path: { organization_id: organizationId, workflow_id: row.id },
					})
				}
			/>

			<WorkflowEditorDialog
				open={draft !== null}
				editingName={
					draft?.id
						? (rows.find((row) => row.id === draft.id)?.name ?? null)
						: null
				}
				values={draft?.values ?? EMPTY_WORKFLOW_FORM}
				isPending={isPending}
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
