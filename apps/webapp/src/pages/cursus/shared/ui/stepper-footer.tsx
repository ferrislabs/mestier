import { Button } from '#/components/ui/button'
import { DialogFooter } from '#/components/ui/dialog'

export interface StepperFooterProps {
	isFirst: boolean
	isLast: boolean
	submitLabel: string
	isPending: boolean
	onCancel: () => void
	onBack: () => void
	onFirstBack?: () => void
}

export function StepperFooter({
	isFirst,
	isLast,
	submitLabel,
	isPending,
	onCancel,
	onBack,
	onFirstBack,
}: StepperFooterProps) {
	return (
		<DialogFooter>
			{isFirst && !onFirstBack ? (
				<Button key="cancel" type="button" variant="ghost" onClick={onCancel}>
					Annuler
				</Button>
			) : (
				<Button
					key="back"
					type="button"
					variant="ghost"
					onClick={isFirst ? onFirstBack : onBack}
				>
					Précédent
				</Button>
			)}
			{isLast ? (
				<Button key="submit" type="submit" disabled={isPending}>
					{submitLabel}
				</Button>
			) : (
				<Button key="next" type="submit">
					Suivant
				</Button>
			)}
		</DialogFooter>
	)
}
