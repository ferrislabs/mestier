import { Link } from '@tanstack/react-router'
import {
	Loader2,
	MoreHorizontal,
	PlaySquare,
	Trash2,
	Workflow,
} from 'lucide-react'
import { CreateButton, TextField } from '#/components/reference-table'
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
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Switch } from '#/components/ui/switch'
import { buildOrgPath } from '#/modules/org-path'
import {
	RUN_STATUS_LABEL,
	RUN_STATUS_TONE,
} from '#/pages/settings/lib/automation'

export interface WorkflowFormValues {
	name: string
	description: string
}

interface FormBinding<T> {
	values: T
	isPending: boolean
	onChange: (patch: Partial<T>) => void
	onSubmit: () => void
}

export interface WorkflowRow {
	id: string
	name: string
	description: string | null
	enabled: boolean
	/** `null` when the workflow has never run yet. */
	lastRunStatus: string | null
	/** `null` alongside `lastRunStatus`. */
	lastRunAt: string | null
}

export interface WorkflowListUIProps {
	organizationName: string
	organizationSlug: string
	isLoading: boolean
	error: string | null
	workflows: WorkflowRow[]
	createForm: FormBinding<WorkflowFormValues>
	togglingId: string | null
	onToggleEnabled: (workflow: WorkflowRow) => void
	deletingId: string | null
	onDelete: (workflow: WorkflowRow) => void
}

export function WorkflowListUI({
	organizationName,
	organizationSlug,
	isLoading,
	error,
	workflows,
	createForm,
	togglingId,
	onToggleEnabled,
	deletingId,
	onDelete,
}: WorkflowListUIProps) {
	const activeCount = workflows.filter((workflow) => workflow.enabled).length

	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Workflows"
				description="Déclenchez des actions automatiquement à partir des événements de l’organisation."
			/>

			<MetricCard
				label="Workflows actifs"
				value={activeCount}
				hint={`sur ${workflows.length} au total`}
				icon={<Workflow className="size-4" />}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<CreateWorkflowSection form={createForm} />

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des workflows…
				</SectionCard>
			) : (
				<WorkflowTable
					data={workflows}
					organizationSlug={organizationSlug}
					togglingId={togglingId}
					onToggleEnabled={onToggleEnabled}
					deletingId={deletingId}
					onDelete={onDelete}
				/>
			)}
		</PageShell>
	)
}

function CreateWorkflowSection({
	form,
}: {
	form: FormBinding<WorkflowFormValues>
}) {
	return (
		<SectionCard>
			<SectionHeader
				title="Nouveau workflow"
				description="Un nom suffit pour commencer — le graphe se construit ensuite dans l’éditeur."
			/>
			<div className="grid grid-cols-1 gap-4 p-5 lg:grid-cols-[1fr_auto] lg:items-end">
				<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
					<TextField
						label="Nom"
						value={form.values.name}
						onChange={(name) => form.onChange({ name })}
					/>
					<TextField
						label="Description"
						value={form.values.description}
						onChange={(description) => form.onChange({ description })}
						placeholder="Optionnel"
					/>
				</div>
				<CreateButton isPending={form.isPending} onClick={form.onSubmit} />
			</div>
		</SectionCard>
	)
}

interface WorkflowTableProps {
	data: WorkflowRow[]
	organizationSlug: string
	togglingId: string | null
	onToggleEnabled: (workflow: WorkflowRow) => void
	deletingId: string | null
	onDelete: (workflow: WorkflowRow) => void
}

function WorkflowTable({
	data,
	organizationSlug,
	togglingId,
	onToggleEnabled,
	deletingId,
	onDelete,
}: WorkflowTableProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Workflows (${data.length})`}
				description="Statut, dernière exécution et accès à l’éditeur ou à l’historique."
			/>
			<div className="overflow-x-auto">
				<table className="w-full min-w-[720px] border-collapse text-sm">
					<thead>
						<tr className="border-b bg-muted/50">
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Nom
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Statut
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Dernier run
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								<span className="sr-only">Actions</span>
							</th>
						</tr>
					</thead>
					<tbody>
						{data.length === 0 ? (
							<tr>
								<td colSpan={4} className="px-5 py-12 text-center">
									<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
										<p className="font-medium">Aucun workflow pour le moment</p>
										<p className="text-sm text-muted-foreground">
											Créez votre premier workflow pour automatiser une action à
											partir d’un événement de l’organisation.
										</p>
									</div>
								</td>
							</tr>
						) : (
							data.map((workflow) => (
								<tr
									key={workflow.id}
									className="group border-b transition hover:bg-muted/35 last:border-b-0"
								>
									<td className="px-5 py-3 align-middle">
										<p className="truncate font-medium">{workflow.name}</p>
										{workflow.description ? (
											<p className="mt-0.5 truncate text-xs text-muted-foreground">
												{workflow.description}
											</p>
										) : null}
									</td>
									<td className="px-5 py-3 align-middle">
										<div className="flex items-center gap-2">
											<Switch
												checked={workflow.enabled}
												disabled={togglingId === workflow.id}
												onCheckedChange={() => onToggleEnabled(workflow)}
												aria-label={
													workflow.enabled
														? `Désactiver ${workflow.name}`
														: `Activer ${workflow.name}`
												}
											/>
											<StatusBadge
												tone={workflow.enabled ? 'success' : 'neutral'}
											>
												{workflow.enabled ? 'Activé' : 'Désactivé'}
											</StatusBadge>
										</div>
									</td>
									<td className="px-5 py-3 align-middle">
										{workflow.lastRunStatus ? (
											<StatusBadge
												tone={
													RUN_STATUS_TONE[workflow.lastRunStatus] ?? 'neutral'
												}
											>
												{RUN_STATUS_LABEL[workflow.lastRunStatus] ??
													workflow.lastRunStatus}
											</StatusBadge>
										) : (
											<span className="text-muted-foreground italic">
												Jamais exécuté
											</span>
										)}
									</td>
									<td className="px-5 py-3 align-middle">
										<RowActions
											workflow={workflow}
											organizationSlug={organizationSlug}
											isDeleting={deletingId === workflow.id}
											onDelete={() => onDelete(workflow)}
										/>
									</td>
								</tr>
							))
						)}
					</tbody>
				</table>
			</div>
		</SectionCard>
	)
}

interface RowActionsProps {
	workflow: WorkflowRow
	organizationSlug: string
	isDeleting: boolean
	onDelete: () => void
}

function RowActions({
	workflow,
	organizationSlug,
	isDeleting,
	onDelete,
}: RowActionsProps) {
	return (
		<AlertDialog>
			<div className="flex justify-end opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<button
							type="button"
							className="inline-flex size-8 items-center justify-center rounded-md hover:bg-muted"
						>
							<MoreHorizontal className="size-4" />
							<span className="sr-only">Actions</span>
						</button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end">
						<DropdownMenuItem asChild>
							<Link
								to={buildOrgPath(organizationSlug, '/automation/$workflowId')}
								params={{ workflowId: workflow.id }}
							>
								<Workflow />
								Ouvrir l’éditeur
							</Link>
						</DropdownMenuItem>
						<DropdownMenuItem asChild>
							<Link
								to={buildOrgPath(
									organizationSlug,
									'/automation/$workflowId/runs',
								)}
								params={{ workflowId: workflow.id }}
							>
								<PlaySquare />
								Voir les runs
							</Link>
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
						Ce workflow et son historique de runs seront supprimés. Cette action
						est irréversible.
					</AlertDialogDescription>
				</AlertDialogHeader>
				<AlertDialogFooter>
					<AlertDialogCancel>Annuler</AlertDialogCancel>
					<AlertDialogAction onClick={onDelete} disabled={isDeleting}>
						{isDeleting ? <Loader2 className="animate-spin" /> : null}
						Supprimer
					</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}
