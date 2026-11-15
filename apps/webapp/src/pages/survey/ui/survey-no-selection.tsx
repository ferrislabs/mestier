import { ClipboardList } from 'lucide-react'

export function SurveyNoSelection() {
	return (
		<div className="flex flex-1 items-center justify-center p-6">
			<div className="grid max-w-sm justify-items-center gap-3 rounded-xl border border-dashed bg-card px-6 py-12 text-center">
				<span
					aria-hidden
					className="flex size-12 items-center justify-center rounded-full bg-muted text-muted-foreground"
				>
					<ClipboardList className="size-6" />
				</span>
				<p className="font-medium">Aucun questionnaire sélectionné</p>
				<p className="text-muted-foreground text-sm">
					Choisissez un questionnaire dans la liste pour le modifier, ou
					créez-en un nouveau.
				</p>
			</div>
		</div>
	)
}
