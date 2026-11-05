import { X } from 'lucide-react'
import { IconButton } from '#/pages/chat/ui/icon-button'

export interface ThreadPanelHeaderUIProps {
	replyCount: number
	onClose: () => void
}

export function ThreadPanelHeaderUI({
	replyCount,
	onClose,
}: ThreadPanelHeaderUIProps) {
	return (
		<header className="flex min-h-12 shrink-0 items-center justify-between gap-3 border-border border-b bg-background px-4 py-2">
			<div>
				<h2 className="font-semibold text-base">Fil de discussion</h2>
				<p className="text-xs text-muted-foreground">
					{replyCount === 0
						? 'Aucune réponse'
						: replyCount === 1
							? '1 réponse'
							: `${replyCount} réponses`}
				</p>
			</div>
			<IconButton label="Fermer le fil" icon={X} onClick={onClose} />
		</header>
	)
}
