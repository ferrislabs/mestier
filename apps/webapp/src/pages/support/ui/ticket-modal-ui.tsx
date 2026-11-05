import type { ReactNode } from 'react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
} from '#/components/ui/dialog'

export function TicketModalUI({
	onClose,
	children,
}: {
	onClose: () => void
	children: ReactNode
}) {
	return (
		<Dialog
			open
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		>
			<DialogContent
				showCloseButton={false}
				onEscapeKeyDown={(event) => {
					const target = event.target
					if (
						(target instanceof HTMLTextAreaElement ||
							target instanceof HTMLInputElement) &&
						target.value.trim() !== ''
					)
						event.preventDefault()
				}}
				className="flex h-[min(56rem,calc(100dvh-3rem))] w-[calc(100vw-3rem)] max-w-6xl flex-col gap-0 overflow-hidden p-0 sm:max-w-6xl"
			>
				<DialogDescription className="sr-only">
					Conversation, notes internes et propriétés du ticket.
				</DialogDescription>
				{children}
			</DialogContent>
		</Dialog>
	)
}
