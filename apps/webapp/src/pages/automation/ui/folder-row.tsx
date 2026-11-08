import { Link } from '@tanstack/react-router'
import { Ellipsis, Folder, FolderInput, Pencil, Trash2 } from 'lucide-react'
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
import { Button, buttonVariants } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import { projectPath } from '#/pages/automation/lib/automation-paths'
import type { WorkflowFolder } from '#/pages/automation/lib/folders'
import { useDropTarget } from '#/pages/automation/ui/use-drop-target'

export function FolderRow({
	folder,
	workflowCount,
	organizationSlug,
	onRename,
	onMove,
	onDelete,
	draggable,
	dragging,
	canDrop,
	onDragStart,
	onDragEnd,
	onDropItem,
}: {
	folder: WorkflowFolder
	workflowCount: number
	organizationSlug: string
	onRename: (folder: WorkflowFolder) => void
	onMove: (folder: WorkflowFolder) => void
	onDelete: (folder: WorkflowFolder) => void
	draggable: boolean
	dragging: boolean
	canDrop: boolean
	onDragStart: (folder: WorkflowFolder) => void
	onDragEnd: () => void
	onDropItem: (folder: WorkflowFolder) => void
}) {
	const drop = useDropTarget({
		accepts: canDrop,
		onDrop: () => onDropItem(folder),
	})

	return (
		<li
			draggable={draggable}
			onDragStart={(event) => {
				event.dataTransfer.effectAllowed = 'move'
				event.dataTransfer.setData('text/plain', folder.id)
				onDragStart(folder)
			}}
			onDragEnd={onDragEnd}
			{...drop.handlers}
			className={cn(
				'relative flex items-center gap-3 px-4 py-3 transition-colors hover:bg-muted/50',
				dragging && 'opacity-40',
				canDrop && 'outline-dashed outline-1 -outline-offset-2 outline-border',
				drop.over && 'bg-accent/60 outline-ring',
			)}
		>
			<Folder
				className="size-5 shrink-0 text-muted-foreground"
				strokeWidth={1.6}
				aria-hidden
			/>
			<div className="min-w-0 flex-1">
				<Link
					to={buildOrgPath(organizationSlug, projectPath(folder.id))}
					draggable={false}
					className="block truncate font-medium text-base outline-none after:absolute after:inset-0 focus-visible:after:ring-2 focus-visible:after:ring-ring focus-visible:after:ring-inset"
				>
					{folder.name}
				</Link>
				<p className="mt-0.5 text-muted-foreground text-sm">
					{workflowCount === 0
						? 'Aucun workflow'
						: `${workflowCount} workflow${workflowCount > 1 ? 's' : ''}`}
				</p>
			</div>
			<RequirePermission permission="MANAGE_AUTOMATION">
				<div className="relative z-10">
					<AlertDialog>
						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button
									variant="ghost"
									size="icon-sm"
									aria-label={`Dossier « ${folder.name} » : actions`}
								>
									<Ellipsis />
								</Button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="end">
								<DropdownMenuItem onSelect={() => onRename(folder)}>
									<Pencil />
									Renommer
								</DropdownMenuItem>
								<DropdownMenuItem onSelect={() => onMove(folder)}>
									<FolderInput />
									Déplacer
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
								<AlertDialogTitle>
									Supprimer le dossier « {folder.name} » ?
								</AlertDialogTitle>
								<AlertDialogDescription>
									Seul un dossier vide peut être supprimé. Déplacez d’abord ses
									workflows et ses sous-dossiers.
								</AlertDialogDescription>
							</AlertDialogHeader>
							<AlertDialogFooter>
								<AlertDialogCancel>Annuler</AlertDialogCancel>
								<AlertDialogAction
									className={buttonVariants({ variant: 'destructive' })}
									onClick={() => onDelete(folder)}
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
