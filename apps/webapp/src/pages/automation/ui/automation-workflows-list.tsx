import type { ColumnDef } from '@tanstack/react-table'
import { Loader2, MoreHorizontal, Plus, Trash2, Workflow } from 'lucide-react'
import { useMemo } from 'react'
import { ReferenceTable, RowIdentity } from '#/components/reference-table'
import { RequirePermission } from '#/components/require-permission'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
	AlertDialogTrigger,
} from '#/components/ui/alert-dialog'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import {
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
} from '#/pages/settings/lib/automation'

export interface WorkflowLastRun {
	status: string
	at: string
}

export interface WorkflowRow {
	id: string
	name: string
	description: string | null
	enabled: boolean
	lastRun: WorkflowLastRun | null
}

export interface AutomationWorkflowsListProps {
	organizationName: string
	workflows: WorkflowRow[]
	isLoading: boolean
	error: string | null
	onCreate: () => void
	onRename: (workflow: WorkflowRow) => void
	onToggleEnabled: (workflow: WorkflowRow) => void
	onDelete: (workflow: WorkflowRow) => void
}

export function AutomationWorkflowsList({
	organizationName,
	workflows,
	isLoading,
	error,
	onCreate,
	onRename,
	onToggleEnabled,
	onDelete,
}: AutomationWorkflowsListProps) {
	const columns = useMemo<ColumnDef<WorkflowRow>[]>(
		() => [
			{
				id: 'name',
				header: 'Workflow',
				cell: ({ row }) => (
					<RowIdentity title={row.original.name} id={row.original.id} />
				),
			},
			{
				id: 'description',
				header: 'Description',
				cell: ({ row }) =>
					row.original.description ? (
						<span className="truncate">{row.original.description}</span>
					) : (
						<span className="text-muted-foreground italic">Aucune</span>
					),
			},
			{
				id: 'enabled',
				header: 'Statut',
				cell: ({ row }) => (
					<StatusBadge tone={row.original.enabled ? 'success' : 'neutral'}>
						{row.original.enabled ? 'Activé' : 'Désactivé'}
					</StatusBadge>
				),
			},
			{
				id: 'lastRun',
				header: 'Dernière exécution',
				cell: ({ row }) => {
					const lastRun = row.original.lastRun
					if (!lastRun) {
						return (
							<span className="text-muted-foreground italic">
								Jamais exécuté
							</span>
						)
					}
					return (
						<div className="flex flex-col gap-1">
							<StatusBadge tone={RUN_STATUS_TONE[lastRun.status] ?? 'neutral'}>
								{RUN_STATUS_LABEL[lastRun.status] ?? lastRun.status}
							</StatusBadge>
							<span className="text-xs text-muted-foreground">
								{lastRun.at}
							</span>
						</div>
					)
				},
			},
			{
				id: 'actions',
				header: () => <span className="sr-only">Actions</span>,
				cell: ({ row }) => (
					<WorkflowRowActions
						workflow={row.original}
						onRename={() => onRename(row.original)}
						onToggleEnabled={() => onToggleEnabled(row.original)}
						onDelete={() => onDelete(row.original)}
					/>
				),
			},
		],
		[onRename, onToggleEnabled, onDelete],
	)

	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Automatisation"
				description="Un workflow automatise une suite d’actions déclenchées par un événement de l’organisation — une facture Odoo créée, un client notifié."
				actions={
					<RequirePermission permission="MANAGE_AUTOMATION">
						<Button onClick={onCreate}>
							<Plus />
							Nouveau workflow
						</Button>
					</RequirePermission>
				}
			/>

			<section className="grid grid-cols-2 gap-4 lg:grid-cols-3">
				<MetricCard
					label="Workflows"
					value={workflows.length}
					icon={<Workflow className="size-4" />}
				/>
			</section>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des workflows…
				</SectionCard>
			) : (
				<ReferenceTable
					title={`Workflows (${workflows.length})`}
					description="Le déclencheur de chaque workflow se choisit une fois ouvert."
					emptyTitle="Aucun workflow"
					emptyDescription="Un workflow automatise une suite d’actions déclenchées par un événement de l’organisation — créez le premier pour commencer."
					data={workflows}
					columns={columns}
				/>
			)}
		</PageShell>
	)
}

interface WorkflowRowActionsProps {
	workflow: WorkflowRow
	onRename: () => void
	onToggleEnabled: () => void
	onDelete: () => void
}

function WorkflowRowActions({
	workflow,
	onRename,
	onToggleEnabled,
	onDelete,
}: WorkflowRowActionsProps) {
	return (
		<RequirePermission permission="MANAGE_AUTOMATION">
			<AlertDialog>
				<div className="flex justify-end opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button size="icon-sm" variant="ghost">
								<MoreHorizontal />
								<span className="sr-only">Actions</span>
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							<DropdownMenuItem onClick={onRename}>Modifier</DropdownMenuItem>
							<DropdownMenuItem onClick={onToggleEnabled}>
								{workflow.enabled ? 'Désactiver' : 'Activer'}
							</DropdownMenuItem>
							<DropdownMenuSeparator />
							<AlertDialogTrigger asChild>
								<DropdownMenuItem
									variant="destructive"
									onSelect={(event) => event.preventDefault()}
								>
									<Trash2 />
									Supprimer
								</DropdownMenuItem>
							</AlertDialogTrigger>
						</DropdownMenuContent>
					</DropdownMenu>
				</div>
				<AlertDialogContent>
					<AlertDialogHeader>
						<AlertDialogTitle>Supprimer {workflow.name} ?</AlertDialogTitle>
						<AlertDialogDescription>
							Ce workflow sera supprimé définitivement. Cette action est
							irréversible.
						</AlertDialogDescription>
					</AlertDialogHeader>
					<AlertDialogFooter>
						<AlertDialogCancel>Annuler</AlertDialogCancel>
						<AlertDialogAction onClick={onDelete}>Supprimer</AlertDialogAction>
					</AlertDialogFooter>
				</AlertDialogContent>
			</AlertDialog>
		</RequirePermission>
	)
}
