import { useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Label } from '#/components/ui/label'
import { absenceFailureOf } from '#/pages/hr/lib/rh/absence-wire'
import { absenceDays, findMember, findType } from '#/pages/hr/lib/rh/absences'
import { describeSpan, formatDayCount } from '#/pages/hr/lib/rh/absences-view'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { Absence } from '#/pages/hr/lib/rh/types'
import { useRhState } from '#/pages/hr/ui/rh/shared/rh-context'

export function RejectDialog({
	absence,
	open,
	onOpenChange,
	onConfirm,
}: {
	absence: Absence | null
	open: boolean
	onOpenChange: (open: boolean) => void
	onConfirm: (absence: Absence, comment: string) => Promise<void>
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				{absence && (
					<RejectForm
						absence={absence}
						onCancel={() => onOpenChange(false)}
						onConfirm={(comment) => onConfirm(absence, comment)}
					/>
				)}
			</DialogContent>
		</Dialog>
	)
}

function RejectForm({
	absence,
	onCancel,
	onConfirm,
}: {
	absence: Absence
	onCancel: () => void
	onConfirm: (comment: string) => Promise<void>
}) {
	const state = useRhState()
	const fieldId = useId()
	const [comment, setComment] = useState('')
	const [error, setError] = useState<string | null>(null)
	const [isPending, setIsPending] = useState(false)
	const blank = comment.trim() === ''

	async function submit() {
		if (isPending) return
		if (blank) {
			setError('Un refus doit être motivé par un commentaire.')
			return
		}
		setIsPending(true)
		setError(null)
		try {
			await onConfirm(comment)
		} catch (failure) {
			setError(absenceFailureOf(failure).message)
		} finally {
			setIsPending(false)
		}
	}

	const member = findMember(state, absence.memberId)
	const type = findType(state, absence.typeId)
	const days = member && type ? absenceDays(absence, member, type) : 0
	return (
		<form
			className="grid gap-4"
			onSubmit={(event) => {
				event.preventDefault()
				void submit()
			}}
		>
			<DialogHeader>
				<DialogTitle>
					Refuser la demande{member ? ` de ${fullName(member)}` : ''}
				</DialogTitle>
				<DialogDescription>
					{`${type?.name ?? 'Absence'}, ${describeSpan(absence)} (${formatDayCount(days)}).`}
				</DialogDescription>
			</DialogHeader>
			<div className="grid gap-1.5">
				<Label htmlFor={fieldId}>Commentaire</Label>
				<textarea
					id={fieldId}
					rows={3}
					value={comment}
					aria-required
					aria-invalid={error ? true : undefined}
					onChange={(event) => {
						setError(null)
						setComment(event.target.value)
					}}
					placeholder="Explique ce choix, par exemple une période de forte activité."
					className="w-full resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30"
				/>
				{error && (
					<p role="alert" className="text-sm text-state-alert">
						{error}
					</p>
				)}
				<p className="text-muted-foreground text-xs">
					Le commentaire est conservé avec la décision dans l’historique.
				</p>
			</div>
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onCancel}>
					Garder la demande
				</Button>
				<Button type="submit" variant="destructive" disabled={isPending}>
					Refuser la demande
				</Button>
			</DialogFooter>
		</form>
	)
}
