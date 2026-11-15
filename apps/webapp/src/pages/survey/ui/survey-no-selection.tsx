import { ClipboardList } from 'lucide-react'

export function SurveyNoSelection() {
	return (
		<div className="flex flex-1 p-6">
			<div className="flex flex-1 flex-col items-center justify-center gap-4 rounded-xl border border-dashed bg-card px-6 py-16 text-center">
				<span
					aria-hidden
					className="flex size-20 items-center justify-center rounded-full bg-muted text-muted-foreground"
				>
					<ClipboardList className="size-10" />
				</span>
				<p className="font-semibold text-xl">Aucun questionnaire sélectionné</p>
				<p className="max-w-md text-base text-muted-foreground">
					Choisissez un questionnaire dans la liste pour le modifier, ou
					créez-en un nouveau.
				</p>
			</div>
		</div>
	)
}
