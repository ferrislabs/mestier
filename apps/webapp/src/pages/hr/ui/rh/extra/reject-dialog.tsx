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
import { findMember } from '#/pages/hr/lib/rh/absences'
import { formatLong } from '#/pages/hr/lib/rh/dates'
import {
	describeSpan,
	extraMinutes,
	formatDuration,
} from '#/pages/hr/lib/rh/extra'
import { extraFailureOf } from '#/pages/hr/lib/rh/extra-wire'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { ExtraPeriod } from '#/pages/hr/lib/rh/types'
import { useRhState } from '#/pages/hr/ui/rh/shared/rh-context'

export function RejectDialog({
	extra,
	open,
	onOpenChange,
	onConfirm,
}: {
	extra: ExtraPeriod | null
	open: boolean
	onOpenChange: (open: boolean) => void
	onConfirm: (extra: ExtraPeriod, comment: string) => Promise<void>
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				{extra && (
					<RejectForm
						extra={extra}
						onCancel={() => onOpenChange(false)}
						onConfirm={(comment) => onConfirm(extra, comment)}
					/>
				)}
			</DialogContent>
		</Dialog>
	)
}

function RejectForm({
	extra,
	onCancel,
	onConfirm,
}: {
	extra: ExtraPeriod
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
			setError(extraFailureOf(failure).message)
		} finally {
			setIsPending(false)
		}
	}

	const member = findMember(state, extra.memberId)
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
					Refuser la déclaration{member ? ` de ${fullName(member)}` : ''}
				</DialogTitle>
				<DialogDescription>
					{`${formatLong(extra.date)}, ${describeSpan(extra)} (${formatDuration(extraMinutes(extra))}).`}
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
					placeholder="Explique ce choix, par exemple une commande non validée."
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
					Garder la déclaration
				</Button>
				<Button type="submit" variant="destructive" disabled={isPending}>
					Refuser la déclaration
				</Button>
			</DialogFooter>
		</form>
	)
}
