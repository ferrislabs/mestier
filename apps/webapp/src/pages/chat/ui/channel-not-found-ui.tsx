import { Hash, SearchX } from 'lucide-react'
import { Button } from '#/components/ui/button'

export function ChannelNotFoundUI({ onBack }: { onBack: () => void }) {
	return (
		<div className="flex min-h-0 flex-1 items-center justify-center p-6">
			<div className="max-w-sm text-center">
				<span className="mx-auto mb-4 flex size-12 items-center justify-center rounded-xl bg-muted text-muted-foreground">
					<SearchX aria-hidden className="size-6" />
				</span>
				<h1 className="font-semibold text-lg">Ce canal est introuvable</h1>
				<p className="mt-1 text-muted-foreground text-sm">
					Ce canal n’existe pas, a été archivé ou n’est plus accessible.
					Vérifiez l’adresse ou retournez à la liste des canaux.
				</p>
				<Button type="button" onClick={onBack} className="mt-5">
					<Hash aria-hidden />
					Retour aux canaux
				</Button>
			</div>
		</div>
	)
}
