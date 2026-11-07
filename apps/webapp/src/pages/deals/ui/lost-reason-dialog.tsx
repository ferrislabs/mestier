import { useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import type { Deal } from '#/pages/deals/lib/deal-types'
import { LOST_REASONS } from '#/pages/deals/lib/pipeline'

export function LostReasonDialog({
	deal,
	onCancel,
	onConfirm,
}: {
	deal: Deal
	onCancel: () => void
	onConfirm: (reason: string) => void
}) {
	const fieldId = useId()
	const [reason, setReason] = useState('')
	const trimmed = reason.trim()
	return (
		<Dialog open onOpenChange={(open) => !open && onCancel()}>
			<DialogContent>
				<form
					noValidate
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						if (trimmed !== '') onConfirm(trimmed)
					}}
				>
					<DialogHeader>
						<DialogTitle>Marquer comme perdue</DialogTitle>
						<DialogDescription>
							{`Pourquoi « ${deal.title} » est-elle perdue ? Le motif est conservé sur la fiche de l’opportunité.`}
						</DialogDescription>
					</DialogHeader>
					<fieldset className="grid gap-2">
						<legend className="mb-1 font-medium text-sm">
							Motifs courants
						</legend>
						<div className="flex flex-wrap gap-2">
							{LOST_REASONS.map((preset) => (
								<Button
									key={preset}
									type="button"
									size="sm"
									variant={trimmed === preset ? 'default' : 'outline'}
									aria-pressed={trimmed === preset}
									onClick={() => setReason(preset)}
								>
									{preset}
								</Button>
							))}
						</div>
					</fieldset>
					<FormField id={fieldId} label="Motif" required>
						<Input
							id={fieldId}
							value={reason}
							onChange={(event) => setReason(event.target.value)}
							autoComplete="off"
							placeholder="Ou saisis un autre motif"
						/>
					</FormField>
					<DialogFooter>
						<Button type="button" variant="outline" onClick={onCancel}>
							Annuler
						</Button>
						<Button
							type="submit"
							variant="destructive"
							disabled={trimmed === ''}
						>
							Marquer comme perdue
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
