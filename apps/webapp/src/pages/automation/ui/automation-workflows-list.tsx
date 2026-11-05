import { Link } from '@tanstack/react-router'
import { Loader2, Plus } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import {
	newWorkflowPath,
	workflowsPath,
} from '#/pages/automation/lib/automation-paths'
import type { WorkflowFolder } from '#/pages/automation/lib/folders'
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
	onRename: (workflow: WorkflowRow) => void
	onToggleEnabled: (workflow: WorkflowRow) => void
	onDuplicate: (workflow: WorkflowRow) => void
	onExport: (workflow: WorkflowRow) => void
	onArchive: (workflow: WorkflowRow) => void
	onDelete: (workflow: WorkflowRow) => void
}

export function AutomationWorkflowsList({
	organizationSlug,
	workflows,
	isLoading,
	error,
	folder,
	folderPath,
	folderMissing,
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
	onRename,
	onToggleEnabled,
	onDuplicate,
	onExport,
	onArchive,
	onDelete,
}: AutomationWorkflowsListProps) {
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
						<Button asChild>
							<Link to={buildOrgPath(organizationSlug, newWorkflowPath())}>
								<Plus />
								Nouveau workflow
							</Link>
						</Button>
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
			{subfolders.length > 0 && (
				<ul
					aria-label="Sous-dossiers"
					className="mb-3 divide-y divide-border overflow-hidden rounded-lg border border-border bg-card"
				>
					{subfolders.map((subfolder) => (
						<FolderRow
							key={subfolder.folder.id}
							folder={subfolder.folder}
							workflowCount={subfolder.workflowCount}
							organizationSlug={organizationSlug}
						/>
					))}
				</ul>
			)}
			{isLoading ? (
				<div className="flex min-h-40 items-center justify-center gap-3 rounded-lg border border-border bg-card p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des workflows…
				</div>
			) : !hasWorkflowsInScope ? (
				folder ? (
					<p className="rounded-lg border border-border border-dashed bg-card px-4 py-8 text-center text-muted-foreground text-sm">
						Ce projet ne contient encore aucun workflow.
					</p>
				) : (
					<WorkflowListNoneYet />
				)
			) : workflows.length === 0 ? (
				<WorkflowListEmpty
					query={query}
					tag={tag === ALL_TAGS ? null : tag}
					onReset={onResetFilters}
				/>
			) : (
				<ul className="divide-y divide-border overflow-hidden rounded-lg border border-border bg-card">
					{workflows.map((workflow) => (
						<WorkflowListRow
							key={workflow.id}
							workflow={workflow}
							organizationSlug={organizationSlug}
							onRename={onRename}
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
