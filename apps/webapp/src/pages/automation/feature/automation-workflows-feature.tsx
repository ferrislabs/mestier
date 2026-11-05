import { useEffect, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationRuns,
	useAutomationWorkflows,
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
	id: string
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
	const [query, setQuery] = useState('')
	const [tag, setTag] = useState(ALL_TAGS)
	const [sort, setSort] = useState<WorkflowSortKey>('updated')

	const workflows = useAutomationWorkflows(organizationId)
	const runs = useAutomationRuns(organizationId)
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

	const error =
		workflows.error ??
		runs.error ??
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
