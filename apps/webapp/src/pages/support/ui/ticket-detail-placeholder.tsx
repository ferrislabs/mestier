import { MessagesSquare } from 'lucide-react'

export function TicketDetailPlaceholder() {
	return (
		<div className="flex h-full flex-col items-center justify-center gap-3 px-8 text-center">
			<span className="flex size-14 items-center justify-center rounded-full bg-muted text-muted-foreground">
				<MessagesSquare className="size-7" strokeWidth={1.5} aria-hidden />
			</span>
			<div>
				<p className="font-medium text-base">Sélectionnez un ticket</p>
				<p className="mt-1 max-w-sm text-muted-foreground text-sm">
					Choisissez un ticket dans la liste pour lire la conversation, y
					répondre ou ajouter une note interne.
				</p>
			</div>
		</div>
	)
}
