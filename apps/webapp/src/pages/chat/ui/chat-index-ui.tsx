import { MessageSquarePlus } from 'lucide-react'

export interface ChatIndexUIProps {
	state: 'loading' | 'empty' | 'redirecting'
}

/**
 * The `/chat` landing pane. `redirecting` renders nothing visible — the
 * feature navigates away to the first channel before a frame paints in the
 * common case — kept as its own state so a slow navigation still shows
 * *something* rather than a blank flash.
 */
export function ChatIndexUI({ state }: ChatIndexUIProps) {
	if (state === 'empty') {
		return (
			<div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-muted-foreground">
				<MessageSquarePlus className="size-10" />
				<div>
					<p className="font-medium text-foreground">
						Aucun canal pour le moment
					</p>
					<p className="mt-1 max-w-sm text-sm">
						Créez le premier canal avec le bouton “+” en haut de la barre
						latérale.
					</p>
				</div>
			</div>
		)
	}

	return null
}
