import { Volume2 } from 'lucide-react'

/** A voice channel carries no text: until voice exists, it says so. */
export function VoiceChannelUI() {
	return (
		<div className="flex min-h-0 flex-1 items-center justify-center p-6">
			<div className="max-w-sm text-center">
				<span className="mx-auto mb-4 flex size-12 items-center justify-center rounded-xl bg-muted text-muted-foreground">
					<Volume2 aria-hidden className="size-6" />
				</span>
				<h2 className="font-semibold text-lg">Salon vocal</h2>
				<p className="mt-1 text-muted-foreground text-sm">
					Les échanges à la voix arrivent bientôt. Ce salon n’accepte pas de
					messages texte.
				</p>
			</div>
		</div>
	)
}

/** Shown in place of the composer where the reader can read but not write. */
export function ReadOnlyNoticeUI({ label }: { label: string }) {
	return (
		<p className="border-border border-t bg-card px-4 py-3 text-center text-muted-foreground text-sm">
			{label}
		</p>
	)
}
