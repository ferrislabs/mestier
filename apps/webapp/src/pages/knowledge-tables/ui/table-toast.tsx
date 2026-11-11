import { X } from 'lucide-react'
import { Button } from '#/components/ui/button'

export function TableToast({
	message,
	onDismiss,
}: {
	message: string | null
	onDismiss: () => void
}) {
	if (!message) return null
	return (
		<div
			role="alert"
			className="fixed right-6 bottom-6 z-50 flex max-w-sm items-start gap-2 rounded-lg border border-destructive/30 bg-popover px-4 py-3 text-destructive text-sm shadow-lg"
		>
			<p className="min-w-0 flex-1">{message}</p>
			<Button
				type="button"
				variant="ghost"
				size="icon-sm"
				aria-label="Fermer le message"
				onClick={onDismiss}
			>
				<X aria-hidden />
			</Button>
		</div>
	)
}
