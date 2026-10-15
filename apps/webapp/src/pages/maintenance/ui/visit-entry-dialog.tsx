import { Loader2 } from 'lucide-react'
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
import { Label } from '#/components/ui/label'

export interface VisitEntryTarget {
	lineId: string
	lineLabel: string
	customerName: string
	week: number
	year: number
}

export interface VisitEntryDialogProps {
	/** `null` keeps the dialog closed. */
	target: VisitEntryTarget | null
	day: string
	error: string | null
	isPending: boolean
	onDayChange: (day: string) => void
	onSubmit: () => void
	onClose: () => void
}

/**
 * The day of the month, as on the paper sheet. The week of the clicked cell
 * says which month it is.
 */
export function VisitEntryDialog({
	target,
	day,
	error,
	isPending,
	onDayChange,
	onSubmit,
	onClose,
}: VisitEntryDialogProps) {
	return (
		<Dialog open={target !== null} onOpenChange={(open) => !open && onClose()}>
			<DialogContent className="sm:max-w-sm">
				<form
					className="flex flex-col gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						onSubmit()
					}}
				>
					<DialogHeader>
						<DialogTitle>Saisir une intervention</DialogTitle>
						<DialogDescription>
							{target
								? `${target.lineLabel}, ${target.customerName}, semaine ${target.week}. L'intervention est validée d'office.`
								: null}
						</DialogDescription>
					</DialogHeader>

					<div className="space-y-1">
						<Label htmlFor="visit-day">Jour du mois</Label>
						<Input
							id="visit-day"
							inputMode="numeric"
							autoComplete="off"
							autoFocus
							value={day}
							onChange={(event) => onDayChange(event.target.value)}
							aria-invalid={error ? true : undefined}
							placeholder="21"
						/>
						{error ? (
							<p role="alert" className="text-sm text-destructive">
								{error}
							</p>
						) : null}
					</div>

					<DialogFooter>
						<Button type="button" variant="outline" onClick={onClose}>
							Annuler
						</Button>
						<Button type="submit" disabled={isPending || !day.trim()}>
							{isPending ? <Loader2 className="animate-spin" /> : null}
							Enregistrer
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
