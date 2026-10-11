import { Loader2, Trash2 } from 'lucide-react'
import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '#/components/ui/alert-dialog'

interface DeleteDoeDialogProps {
	open: boolean
	title: string
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onConfirm: () => void
}

export function DeleteDoeDialog({
	open,
	title,
	isPending,
	error,
	onOpenChange,
	onConfirm,
}: DeleteDoeDialogProps) {
	return (
		<AlertDialog open={open} onOpenChange={onOpenChange}>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>Supprimer ce DOE ?</AlertDialogTitle>
					<AlertDialogDescription>
						{title
							? `Le DOE « ${title} » sera supprimé. `
							: 'Ce DOE sera supprimé. '}
						La suppression est définitive. Le projet pourra recevoir un nouveau
						DOE. Les photos et documents déjà envoyés ne sont pas supprimés du
						stockage.
					</AlertDialogDescription>
				</AlertDialogHeader>
				{error ? (
					<p role="alert" className="text-sm text-destructive">
						{error}
					</p>
				) : null}
				<AlertDialogFooter>
					<AlertDialogCancel disabled={isPending}>Annuler</AlertDialogCancel>
					<AlertDialogAction
						disabled={isPending}
						className="bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20"
						onClick={(event) => {
							event.preventDefault()
							onConfirm()
						}}
					>
						{isPending ? <Loader2 className="animate-spin" /> : <Trash2 />}
						Supprimer définitivement
					</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}
