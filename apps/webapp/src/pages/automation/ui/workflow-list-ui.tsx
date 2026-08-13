import { Link } from '@tanstack/react-router'
import {
	Loader2,
	MoreHorizontal,
	PlaySquare,
	Plus,
	Trash2,
	Workflow,
} from 'lucide-react'
import { TextField } from '#/components/reference-table'
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
	Card,
	CardAction,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
} from '#/components/ui/card'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
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
	createDialogOpen: boolean
	onOpenCreateDialog: () => void
	onCreateDialogOpenChange: (open: boolean) => void
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
	createDialogOpen,
	onOpenCreateDialog,
	onCreateDialogOpenChange,
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
				actions={
					<Button onClick={onOpenCreateDialog} className="gap-2">
						<Plus className="size-4" />
						Ajouter
					</Button>
				}
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

			<CreateWorkflowDialog
				open={createDialogOpen}
				onOpenChange={onCreateDialogOpenChange}
				form={createForm}
			/>

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des workflows…
				</SectionCard>
			) : (
				<WorkflowGrid
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

function CreateWorkflowDialog({
	open,
	onOpenChange,
	form,
}: {
	open: boolean
	onOpenChange: (open: boolean) => void
	form: FormBinding<WorkflowFormValues>
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<form
					onSubmit={(event) => {
						event.preventDefault()
						form.onSubmit()
					}}
				>
					<DialogHeader>
						<DialogTitle>Nouveau workflow</DialogTitle>
						<DialogDescription>
							Un nom suffit pour commencer — le graphe se construit ensuite dans
							l’éditeur.
						</DialogDescription>
					</DialogHeader>
					<div className="grid grid-cols-1 gap-4 py-4">
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
					<DialogFooter>
						<Button
							type="button"
							variant="ghost"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={form.isPending}>
							{form.isPending ? <Loader2 className="animate-spin" /> : null}
							Créer
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}

interface WorkflowGridProps {
	data: WorkflowRow[]
	organizationSlug: string
	togglingId: string | null
	onToggleEnabled: (workflow: WorkflowRow) => void
	deletingId: string | null
	onDelete: (workflow: WorkflowRow) => void
}

function WorkflowGrid({
	data,
	organizationSlug,
	togglingId,
	onToggleEnabled,
	deletingId,
	onDelete,
}: WorkflowGridProps) {
	if (data.length === 0) {
		return (
			<SectionCard className="flex min-h-56 flex-col items-center justify-center gap-2 p-8 text-center">
				<p className="font-medium">Aucun workflow pour le moment</p>
				<p className="text-sm text-muted-foreground">
					Créez votre premier workflow pour automatiser une action à partir d’un
					événement de l’organisation.
				</p>
			</SectionCard>
		)
	}

	return (
		<div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{data.map((workflow) => (
				<WorkflowCard
					key={workflow.id}
					workflow={workflow}
					organizationSlug={organizationSlug}
					isToggling={togglingId === workflow.id}
					onToggleEnabled={() => onToggleEnabled(workflow)}
					isDeleting={deletingId === workflow.id}
					onDelete={() => onDelete(workflow)}
				/>
			))}
		</div>
	)
}

interface WorkflowCardProps {
	workflow: WorkflowRow
	organizationSlug: string
	isToggling: boolean
	onToggleEnabled: () => void
	isDeleting: boolean
	onDelete: () => void
}

function WorkflowCard({
	workflow,
	organizationSlug,
	isToggling,
	onToggleEnabled,
	isDeleting,
	onDelete,
}: WorkflowCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle className="truncate">{workflow.name}</CardTitle>
				<CardDescription className="line-clamp-2">
					{workflow.description ?? 'Aucune description'}
				</CardDescription>
				<CardAction>
					<RowActions
						workflow={workflow}
						organizationSlug={organizationSlug}
						isDeleting={isDeleting}
						onDelete={onDelete}
					/>
				</CardAction>
			</CardHeader>
			<CardContent className="flex flex-col gap-3">
				<div className="flex items-center gap-2">
					<Switch
						checked={workflow.enabled}
						disabled={isToggling}
						onCheckedChange={onToggleEnabled}
						aria-label={
							workflow.enabled
								? `Désactiver ${workflow.name}`
								: `Activer ${workflow.name}`
						}
					/>
					<StatusBadge tone={workflow.enabled ? 'success' : 'neutral'}>
						{workflow.enabled ? 'Activé' : 'Désactivé'}
					</StatusBadge>
				</div>
				<div>
					{workflow.lastRunStatus ? (
						<StatusBadge
							tone={RUN_STATUS_TONE[workflow.lastRunStatus] ?? 'neutral'}
						>
							{RUN_STATUS_LABEL[workflow.lastRunStatus] ??
								workflow.lastRunStatus}
						</StatusBadge>
					) : (
						<span className="text-sm text-muted-foreground italic">
							Jamais exécuté
						</span>
					)}
				</div>
			</CardContent>
			<CardFooter className="gap-2">
				<Button variant="outline" size="sm" asChild className="flex-1">
					<Link
						to={buildOrgPath(organizationSlug, '/automation/$workflowId')}
						params={{ workflowId: workflow.id }}
					>
						Ouvrir l’éditeur
					</Link>
				</Button>
			</CardFooter>
		</Card>
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
