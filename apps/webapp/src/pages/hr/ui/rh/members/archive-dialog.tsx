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
import {
	directReports,
	findMemberById,
	fullName,
} from '#/pages/hr/lib/rh/members'
import type { IsoDate, Member, RhState } from '#/pages/hr/lib/rh/types'

function reportsSentence(count: number, manager: Member | undefined) {
	const subject =
		count === 1
			? '1 membre de son équipe directe'
			: `${count} membres de son équipe directe`
	if (manager)
		return `${subject} ${count === 1 ? 'passera' : 'passeront'} sous ${fullName(manager)}.`
	return `${subject} ${count === 1 ? 'n’aura' : 'n’auront'} plus de responsable.`
}

function waitingSentence(count: number) {
	return count === 1
		? '1 demande d’absence future en attente sera annulée.'
		: `${count} demandes d’absence futures en attente seront annulées.`
}

export function ArchiveDialog({
	state,
	member,
	today,
	open,
	onOpenChange,
	onConfirm,
	trigger,
}: {
	state: RhState
	member: Member
	today: IsoDate
	open: boolean
	onOpenChange: (open: boolean) => void
	onConfirm: () => void
	trigger: ReactNode
}) {
	const reports = directReports(state, member.id)
	const manager = findMemberById(state, member.managerId)
	const waiting = state.absences.filter(
		(absence) =>
			absence.memberId === member.id &&
			absence.status === 'pending' &&
			absence.startDate > today,
	)
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogTrigger asChild>{trigger}</DialogTrigger>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Archiver {fullName(member)} ?</DialogTitle>
					<DialogDescription>
						Le membre disparaît du planning et des listes actives. Son
						historique, ses soldes et ses documents sont conservés.
					</DialogDescription>
				</DialogHeader>
				<ul className="grid gap-2 text-sm">
					<li className="rounded-md bg-muted/60 px-3 py-2">
						{reports.length > 0
							? reportsSentence(reports.length, manager)
							: 'Personne n’est rattaché directement à ce membre.'}
					</li>
					<li className="rounded-md bg-muted/60 px-3 py-2">
						{waiting.length > 0
							? waitingSentence(waiting.length)
							: 'Aucune demande d’absence future en attente à annuler.'}
					</li>
				</ul>
				<DialogFooter>
					<DialogClose asChild>
						<Button variant="outline">Annuler</Button>
					</DialogClose>
					<Button variant="destructive" onClick={onConfirm}>
						Archiver
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
