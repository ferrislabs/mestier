import { type ComponentProps, useEffect, useState } from 'react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'

const DISARM_AFTER_MS = 4000

export interface DoubleClickConfirmButtonProps
	extends Omit<ComponentProps<typeof Button>, 'onClick' | 'children'> {
	label: string
	confirmLabel?: string
	onConfirm: () => void
}

/** A destructive action asked twice: the first click arms the button, the
 * second one confirms. It disarms by itself after a few seconds. */
export function DoubleClickConfirmButton({
	label,
	confirmLabel = 'Cliquer à nouveau pour confirmer',
	onConfirm,
	...props
}: DoubleClickConfirmButtonProps) {
	const [armed, setArmed] = useState(false)

	useEffect(() => {
		if (!armed) return
		const timer = setTimeout(() => setArmed(false), DISARM_AFTER_MS)
		return () => clearTimeout(timer)
	}, [armed])

	return (
		<Button
			type="button"
			{...props}
			variant={armed ? 'destructive' : props.variant}
			className={cn(
				props.className,
				armed &&
					'bg-destructive text-white hover:bg-destructive/90 hover:text-white',
			)}
			onClick={() => {
				if (!armed) {
					setArmed(true)
					return
				}
				setArmed(false)
				onConfirm()
			}}
			onBlur={() => setArmed(false)}
		>
			{armed ? confirmLabel : label}
		</Button>
	)
}
