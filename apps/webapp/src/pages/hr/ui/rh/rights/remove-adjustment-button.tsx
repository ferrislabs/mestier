import { Check, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import { useConfirmation } from '#/pages/hr/ui/rh/rights/use-confirmation'

export function RemoveAdjustmentButton({
	label,
	onRemove,
}: {
	label: string
	onRemove: () => void
}) {
	const { armed, press, cancel } = useConfirmation()

	return (
		<>
			<Button
				type="button"
				variant="ghost"
				size={armed ? 'xs' : 'icon-xs'}
				aria-label={armed ? undefined : label}
				onClick={() => press(onRemove)}
				onBlur={cancel}
				className={cn(
					'text-muted-foreground',
					armed &&
						'bg-state-alert/12 text-state-alert hover:bg-state-alert/20 hover:text-state-alert',
				)}
			>
				{armed ? <Check /> : <Trash2 />}
				{armed && 'Confirmer la suppression'}
			</Button>
			<output className="sr-only">
				{armed ? 'Cliquez à nouveau pour confirmer la suppression.' : ''}
			</output>
		</>
	)
}
