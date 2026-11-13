import { Button } from '#/components/ui/button'
import { DialogFooter } from '#/components/ui/dialog'

export interface StepperFooterProps {
	isFirst: boolean
	isLast: boolean
	submitLabel: string
	isPending: boolean
	onCancel: () => void
	onBack: () => void
}

export function StepperFooter({
	isFirst,
	isLast,
	submitLabel,
	isPending,
	onCancel,
	onBack,
}: StepperFooterProps) {
	return (
		<DialogFooter>
			{isFirst ? (
				<Button key="cancel" type="button" variant="ghost" onClick={onCancel}>
					Annuler
				</Button>
			) : (
				<Button key="back" type="button" variant="ghost" onClick={onBack}>
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
