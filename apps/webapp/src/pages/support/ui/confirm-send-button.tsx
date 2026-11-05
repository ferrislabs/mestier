import { Check, Send } from 'lucide-react'
import { useEffect } from 'react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import type { Confirmation } from '#/pages/support/ui/use-confirmation'

export function ConfirmSendButton({
	confirmation,
	onSend,
	disabled = false,
}: {
	confirmation: Confirmation
	onSend: () => void
	disabled?: boolean
}) {
	const { armed, press, cancel } = confirmation

	useEffect(() => {
		if (disabled) cancel()
	}, [disabled, cancel])

	return (
		<>
			<Button
				type="button"
				size="sm"
				disabled={disabled}
				onClick={() => press(onSend)}
				onBlur={cancel}
				className={cn(
					armed &&
						'bg-state-today text-primary-foreground hover:bg-state-today/90',
				)}
			>
				{armed ? <Check /> : <Send />}
				{armed ? 'Confirmer l’envoi' : 'Envoyer au client'}
			</Button>
			<output className="sr-only">
				{armed ? 'Cliquez à nouveau pour envoyer le message au client.' : ''}
			</output>
		</>
	)
}
