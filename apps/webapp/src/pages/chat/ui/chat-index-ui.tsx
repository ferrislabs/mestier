import { MessageSquarePlus } from 'lucide-react'

export interface ChatIndexUIProps {
	state: 'loading' | 'empty' | 'redirecting'
}

export function ChatIndexUI({ state }: ChatIndexUIProps) {
	if (state === 'empty') {
		return (
			<div className="flex min-h-0 flex-1 items-center justify-center p-6">
				<div className="max-w-sm text-center">
					<span className="mx-auto mb-4 flex size-12 items-center justify-center rounded-xl bg-muted text-muted-foreground">
						<MessageSquarePlus aria-hidden className="size-6" />
					</span>
					<h1 className="font-semibold text-lg">Aucun canal pour le moment</h1>
					<p className="mt-1 text-muted-foreground text-sm">
						Créez le premier canal avec le bouton “+” en haut de la barre
						latérale.
					</p>
				</div>
			</div>
		)
	}

	return null
}
