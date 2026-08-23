import { X } from 'lucide-react'
import { Button } from '#/components/ui/button'

export interface ThreadPanelHeaderUIProps {
	replyCount: number
	onClose: () => void
}

export function ThreadPanelHeaderUI({
	replyCount,
	onClose,
}: ThreadPanelHeaderUIProps) {
	return (
		<div className="flex items-center justify-between border-b px-4 py-3">
			<div>
				<h2 className="font-semibold text-foreground">Fil de discussion</h2>
				<p className="text-xs text-muted-foreground">
					{replyCount === 0
						? 'Aucune réponse'
						: replyCount === 1
							? '1 réponse'
							: `${replyCount} réponses`}
				</p>
			</div>
			<Button
				type="button"
				variant="ghost"
				size="icon"
				aria-label="Fermer le fil"
				onClick={onClose}
			>
				<X className="size-4" />
			</Button>
		</div>
	)
}
