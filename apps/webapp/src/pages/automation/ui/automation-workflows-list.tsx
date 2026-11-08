import { Link } from '@tanstack/react-router'
import { FolderPlus, Loader2, Plus } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { useHasPermission } from '#/hooks/use-permissions'
import { buildOrgPath } from '#/modules/org-path'
import {
	newWorkflowPath,
	workflowsPath,
} from '#/pages/automation/lib/automation-paths'
import type { DragItem, WorkflowFolder } from '#/pages/automation/lib/folders'
import {
	ALL_TAGS,
	type WorkflowSortKey,
} from '#/pages/automation/lib/workflow-list'
import { FolderBreadcrumb } from '#/pages/automation/ui/folder-breadcrumb'
import { FolderRow } from '#/pages/automation/ui/folder-row'
import {
	WorkflowListEmpty,
	WorkflowListNoneYet,
} from '#/pages/automation/ui/workflow-list-empty'
import { WorkflowListToolbar } from '#/pages/automation/ui/workflow-list-toolbar'
import {
	WorkflowListRow,
	type WorkflowRow,
} from '#/pages/automation/ui/workflow-row'

export type {
	WorkflowLastRun,
	WorkflowRow,
} from '#/pages/automation/ui/workflow-row'

export interface SubfolderRow {
	folder: WorkflowFolder
	workflowCount: number
}

export interface AutomationWorkflowsListProps {
	organizationSlug: string
	workflows: WorkflowRow[]
	isLoading: boolean
	error: string | null
	folder: WorkflowFolder | null
	folderPath: WorkflowFolder[]
	folderMissing: boolean
	folderPending: boolean
	subfolders: SubfolderRow[]
	hasWorkflowsInScope: boolean
	query: string
	onQueryChange: (query: string) => void
	tag: string
	tags: string[]
	onTagChange: (tag: string) => void
	sort: WorkflowSortKey
	onSortChange: (sort: WorkflowSortKey) => void
	onResetFilters: () => void
	dragItem: DragItem | null
	canDropOn: (targetId: string | null) => boolean
	onDragStart: (item: DragItem) => void
	onDragEnd: () => void
	onDropOn: (targetId: string | null) => void
	onCreateFolder: () => void
	onRenameFolder: (folder: WorkflowFolder) => void
	onMoveFolder: (folder: WorkflowFolder) => void
	onDeleteFolder: (folder: WorkflowFolder) => void
	onRename: (workflow: WorkflowRow) => void
	onMove: (workflow: WorkflowRow) => void
	onToggleEnabled: (workflow: WorkflowRow) => void
	onDuplicate: (workflow: WorkflowRow) => void
	onExport: (workflow: WorkflowRow) => void
	onArchive: (workflow: WorkflowRow) => void
	onDelete: (workflow: WorkflowRow) => void
}

function WorkflowsLoading() {
	return (
		<div className="flex min-h-40 items-center justify-center gap-3 rounded-lg border border-border bg-card p-8 text-sm text-muted-foreground">
			<Loader2 className="size-5 animate-spin" />
			Chargement des workflows…
		</div>
	)
}

export function AutomationWorkflowsList({
	organizationSlug,
	workflows,
	isLoading,
	error,
	folder,
	folderPath,
	folderMissing,
	folderPending,
	subfolders,
	hasWorkflowsInScope,
	query,
	onQueryChange,
	tag,
	tags,
	onTagChange,
	sort,
	onSortChange,
	onResetFilters,
	dragItem,
	canDropOn,
	onDragStart,
	onDragEnd,
	onDropOn,
	onCreateFolder,
	onRenameFolder,
	onMoveFolder,
	onDeleteFolder,
	onRename,
	onMove,
	onToggleEnabled,
	onDuplicate,
	onExport,
	onArchive,
	onDelete,
}: AutomationWorkflowsListProps) {
	const canManage = useHasPermission('MANAGE_AUTOMATION')

	if (folderPending) {
		return (
			<PageShell className="gap-0">
				<WorkflowsLoading />
			</PageShell>
		)
	}

	if (folderMissing) {
		return (
			<PageShell className="gap-0">
				<PageHeader
					className="mb-6"
					title="Projet introuvable"
					description="Ce projet n’existe pas ou a été supprimé."
					actions={
						<Button asChild variant="outline">
							<Link to={buildOrgPath(organizationSlug, workflowsPath())}>
								Voir tous les workflows
							</Link>
						</Button>
					}
				/>
			</PageShell>
		)
	}

	return (
		<PageShell className="gap-0">
			{folder && (
				<FolderBreadcrumb
					path={folderPath}
					organizationSlug={organizationSlug}
					canDrop={(targetId) => dragItem !== null && canDropOn(targetId)}
					onDropItem={onDropOn}
				/>
			)}
			<PageHeader
				className="mb-6"
				title={folder ? folder.name : 'Workflows'}
				description={
					folder
						? 'Projet : les workflows et sous-dossiers qu’il contient.'
						: 'Automatisez vos tâches à partir des événements de Mestier.'
				}
				actions={
					<RequirePermission permission="MANAGE_AUTOMATION">
						<div className="flex items-center gap-2">
							<Button variant="outline" onClick={onCreateFolder}>
								<FolderPlus />
								Nouveau dossier
							</Button>
							<Button asChild>
								<Link to={buildOrgPath(organizationSlug, newWorkflowPath())}>
									<Plus />
									Nouveau workflow
								</Link>
							</Button>
						</div>
					</RequirePermission>
				}
			/>
			{error ? (
				<div className="mb-4 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}
			<WorkflowListToolbar
				query={query}
				onQueryChange={onQueryChange}
				tag={tag}
				tags={tags}
				onTagChange={onTagChange}
				sort={sort}
				onSortChange={onSortChange}
			/>
			{isLoading ? (
				<WorkflowsLoading />
			) : subfolders.length === 0 && workflows.length === 0 ? (
				!hasWorkflowsInScope ? (
					folder ? (
						<p className="rounded-lg border border-border border-dashed bg-card px-4 py-8 text-center text-muted-foreground text-sm">
							Ce dossier est vide.
						</p>
					) : (
						<WorkflowListNoneYet />
					)
				) : (
					<WorkflowListEmpty
						query={query}
						tag={tag === ALL_TAGS ? null : tag}
						onReset={onResetFilters}
					/>
				)
			) : (
				<ul
					aria-label="Contenu du dossier"
					className="divide-y divide-border overflow-hidden rounded-lg border border-border bg-card"
				>
					{subfolders.map((subfolder) => (
						<FolderRow
							key={subfolder.folder.id}
							folder={subfolder.folder}
							workflowCount={subfolder.workflowCount}
							organizationSlug={organizationSlug}
							onRename={onRenameFolder}
							onMove={onMoveFolder}
							onDelete={onDeleteFolder}
							draggable={canManage}
							dragging={
								dragItem?.kind === 'folder' &&
								dragItem.id === subfolder.folder.id
							}
							canDrop={dragItem !== null && canDropOn(subfolder.folder.id)}
							onDragStart={(dragged) =>
								onDragStart({
									kind: 'folder',
									id: dragged.id,
									parentId: dragged.parentId,
								})
							}
							onDragEnd={onDragEnd}
							onDropItem={(target) => onDropOn(target.id)}
						/>
					))}
					{workflows.map((workflow) => (
						<WorkflowListRow
							key={workflow.id}
							workflow={workflow}
							organizationSlug={organizationSlug}
							onRename={onRename}
							onMove={onMove}
							draggable={canManage}
							dragging={
								dragItem?.kind === 'workflow' && dragItem.id === workflow.id
							}
							onDragStart={(dragged) =>
								onDragStart({
									kind: 'workflow',
									id: dragged.id,
									parentId: dragged.folderId,
								})
							}
							onDragEnd={onDragEnd}
							onToggleEnabled={onToggleEnabled}
							onDuplicate={onDuplicate}
							onExport={onExport}
							onArchive={onArchive}
							onDelete={onDelete}
						/>
					))}
				</ul>
			)}
		</PageShell>
	)
}
