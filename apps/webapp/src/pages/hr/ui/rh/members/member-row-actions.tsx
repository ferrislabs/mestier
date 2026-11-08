import { Link } from '@tanstack/react-router'
import { Clock, MoreHorizontal, Trash2, UserPlus } from 'lucide-react'
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
import type { AccessState } from '#/pages/hr/types'
import { useRhOrgPath } from '#/pages/hr/ui/rh/shared/rh-context'

export function MemberRowActions({
	memberId,
	memberName,
	access,
	archived,
	onEdit,
	onInvite,
	onDelete,
}: {
	memberId: string
	memberName: string
	access: AccessState
	archived: boolean
	onEdit: () => void
	onInvite: () => void
	onDelete: () => void
}) {
	const orgPath = useRhOrgPath()
	return (
		<AlertDialog>
			<div className="flex justify-end opacity-0 transition focus-within:opacity-100 group-focus-within:opacity-100 group-hover:opacity-100">
				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<Button size="icon-sm" variant="ghost">
							<MoreHorizontal />
							<span className="sr-only">Actions</span>
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end">
						<RequirePermission permission="MANAGE_MEMBERS">
							<DropdownMenuItem onClick={onEdit}>Modifier</DropdownMenuItem>
						</RequirePermission>
						{access === 'none' && !archived ? (
							<DropdownMenuItem onClick={onInvite}>
								<UserPlus />
								Inviter
							</DropdownMenuItem>
						) : null}
						<DropdownMenuItem asChild>
							<Link
								to={orgPath('/rh/membres/$memberId/temps-de-travail')}
								params={{ memberId }}
							>
								<Clock />
								Temps de travail
							</Link>
						</DropdownMenuItem>
						<RequirePermission permission="MANAGE_MEMBERS">
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
						</RequirePermission>
					</DropdownMenuContent>
				</DropdownMenu>
			</div>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>Supprimer {memberName} ?</AlertDialogTitle>
					<AlertDialogDescription>
						Cette personne sera retirée de l’organisation. Cette action est
						irréversible. Pour garder son historique, archivez-la depuis sa
						fiche.
					</AlertDialogDescription>
				</AlertDialogHeader>
				<AlertDialogFooter>
					<AlertDialogCancel>Annuler</AlertDialogCancel>
					<AlertDialogAction onClick={onDelete}>Supprimer</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}
