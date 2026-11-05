import { CircleCheck, X } from 'lucide-react'
import { Button } from '#/components/ui/button'

export function Notice({
	message,
	onDismiss,
}: {
	message: string | null
	onDismiss: () => void
}) {
	return (
		<output
			className={
				message
					? 'mb-4 flex items-start gap-2.5 rounded-md border border-state-progress/30 bg-state-progress/8 py-2 pr-2 pl-3 text-sm'
					: 'sr-only'
			}
		>
			{message && (
				<>
					<CircleCheck
						className="mt-0.5 size-4 shrink-0 text-state-progress"
						aria-hidden
					/>
					<span className="min-w-0 flex-1">{message}</span>
					<Button
						type="button"
						variant="ghost"
						size="icon-xs"
						onClick={onDismiss}
						aria-label="Fermer la notification"
					>
						<X aria-hidden />
					</Button>
				</>
			)}
		</output>
	)
}
