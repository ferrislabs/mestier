import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from '#/components/ui/dialog'
import { CONTRACT_LABELS } from '#/pages/hr/lib/rh/contracts'
import { contractPeriod } from '#/pages/hr/lib/rh/format'
import type { Contract } from '#/pages/hr/lib/rh/types'

export function ContractDeleteDialog({
	contract,
	open,
	onOpenChange,
	onConfirm,
	error = null,
	isPending = false,
	trigger,
}: {
	contract: Contract
	open: boolean
	onOpenChange: (open: boolean) => void
	onConfirm: () => void
	error?: string | null
	isPending?: boolean
	trigger: ReactNode
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogTrigger asChild>{trigger}</DialogTrigger>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Supprimer ce contrat ?</DialogTitle>
					<DialogDescription>
						{CONTRACT_LABELS[contract.type]}, {contractPeriod(contract)}. Sa
						semaine type est supprimée avec lui.
					</DialogDescription>
				</DialogHeader>
				<p className="rounded-md bg-muted/60 px-3 py-2 text-sm">
					Les autres contrats ne changent pas : vérifie qu’aucune période ne
					reste sans contrat.
				</p>
				{error && (
					<p role="alert" className="text-[13px] text-state-alert">
						{error}
					</p>
				)}
				<DialogFooter>
					<DialogClose asChild>
						<Button variant="outline">Annuler</Button>
					</DialogClose>
					<Button
						variant="destructive"
						disabled={isPending}
						onClick={onConfirm}
					>
						Supprimer
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
