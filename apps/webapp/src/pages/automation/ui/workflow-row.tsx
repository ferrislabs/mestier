import { Link } from '@tanstack/react-router'
import {
	Archive,
	Copy,
	Download,
	Ellipsis,
	Folder,
	History,
	Pencil,
	Trash2,
} from 'lucide-react'
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
import { Badge } from '#/components/ui/badge'
import { Button, buttonVariants } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Switch } from '#/components/ui/switch'
import { buildOrgPath } from '#/modules/org-path'
import {
	workflowExecutionsSearch,
	workflowPath,
} from '#/pages/automation/lib/automation-paths'
import { formatAgo, minutesSince } from '#/pages/automation/lib/format'
import { RunStatusLabel } from '#/pages/automation/ui/run-status-label'

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
	updatedAt: string
	tags: readonly string[]
	folderId: string | null
	folderLabel: string | null
}

export function WorkflowListRow({
	workflow,
	organizationSlug,
	onRename,
	onToggleEnabled,
	onDuplicate,
	onExport,
	onArchive,
	onDelete,
}: {
	workflow: WorkflowRow
	organizationSlug: string
	onRename: (workflow: WorkflowRow) => void
	onToggleEnabled: (workflow: WorkflowRow) => void
	onDuplicate: (workflow: WorkflowRow) => void
	onExport: (workflow: WorkflowRow) => void
	onArchive: (workflow: WorkflowRow) => void
	onDelete: (workflow: WorkflowRow) => void
}) {
	return (
		<li className="relative grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 px-4 py-3 transition-colors hover:bg-muted/50 md:grid-cols-[minmax(0,1fr)_200px_200px_auto]">
			<div className="min-w-0">
				<Link
					to={buildOrgPath(organizationSlug, workflowPath(workflow.id))}
					className="block truncate font-medium text-base outline-none after:absolute after:inset-0 focus-visible:after:ring-2 focus-visible:after:ring-ring focus-visible:after:ring-inset"
				>
					{workflow.name}
				</Link>
				<p className="mt-0.5 flex flex-wrap items-center gap-x-3 text-muted-foreground text-sm">
					<span>Modifié {formatAgo(minutesSince(workflow.updatedAt))}</span>
					{workflow.folderLabel && (
						<span className="inline-flex items-center gap-1">
							<Folder className="size-3.5" aria-hidden />
							<span className="sr-only">Projet :</span>
							{workflow.folderLabel}
						</span>
					)}
				</p>
				{workflow.description && (
					<p className="mt-0.5 truncate text-muted-foreground text-sm">
						{workflow.description}
					</p>
				)}
			</div>
			<div className="hidden flex-wrap gap-1 md:flex">
				{workflow.tags.map((tag) => (
					<Badge key={tag} variant="secondary">
						{tag}
					</Badge>
				))}
			</div>
			<div className="hidden md:block">
				{workflow.lastRun ? (
					<RunStatusLabel
						status={workflow.lastRun.status}
						detail={formatAgo(minutesSince(workflow.lastRun.at))}
					/>
				) : (
					<RunStatusLabel status="never" />
				)}
			</div>
			<RequirePermission permission="MANAGE_AUTOMATION">
				<div className="relative z-10 flex items-center gap-2">
					<Switch
						checked={workflow.enabled}
						onCheckedChange={() => onToggleEnabled(workflow)}
						aria-label={`${workflow.enabled ? 'Désactiver' : 'Activer'} le workflow « ${workflow.name} »`}
					/>
					<AlertDialog>
						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button
									variant="ghost"
									size="icon-sm"
									aria-label={`Actions pour « ${workflow.name} »`}
								>
									<Ellipsis />
								</Button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="end">
								<DropdownMenuItem onSelect={() => onDuplicate(workflow)}>
									<Copy />
									Dupliquer
								</DropdownMenuItem>
								<DropdownMenuItem onSelect={() => onExport(workflow)}>
									<Download />
									Exporter
								</DropdownMenuItem>
								<DropdownMenuItem asChild>
									<Link
										to={buildOrgPath(
											organizationSlug,
											workflowPath(workflow.id),
										)}
										search={workflowExecutionsSearch()}
									>
										<History />
										Historique d’exécution
									</Link>
								</DropdownMenuItem>
								<DropdownMenuItem onSelect={() => onRename(workflow)}>
									<Pencil />
									Renommer
								</DropdownMenuItem>
								<DropdownMenuSeparator />
								<DropdownMenuItem
									variant="destructive"
									onSelect={() => onArchive(workflow)}
								>
									<Archive />
									Archiver
								</DropdownMenuItem>
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
								<AlertDialogTitle>
									Supprimer « {workflow.name} » ?
								</AlertDialogTitle>
								<AlertDialogDescription>
									Ce workflow sera supprimé définitivement. Cette action est
									irréversible.
								</AlertDialogDescription>
							</AlertDialogHeader>
							<AlertDialogFooter>
								<AlertDialogCancel>Annuler</AlertDialogCancel>
								<AlertDialogAction
									className={buttonVariants({ variant: 'destructive' })}
									onClick={() => onDelete(workflow)}
								>
									Supprimer
								</AlertDialogAction>
							</AlertDialogFooter>
						</AlertDialogContent>
					</AlertDialog>
				</div>
			</RequirePermission>
		</li>
	)
}
