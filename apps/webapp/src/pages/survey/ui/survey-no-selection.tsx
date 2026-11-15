import { ClipboardList } from 'lucide-react'

export function SurveyNoSelection() {
	return (
		<div className="flex flex-1 items-center justify-center p-6">
			<div className="flex w-full max-w-md flex-col items-center gap-3 rounded-xl border border-dashed px-6 py-10 text-center">
				<span
					aria-hidden
					className="flex size-14 items-center justify-center rounded-full bg-muted text-muted-foreground"
				>
					<ClipboardList className="size-7" />
				</span>
				<p className="font-semibold text-base">
					Aucun questionnaire sélectionné
				</p>
				<p className="text-muted-foreground text-sm">
					Choisissez un questionnaire dans la liste pour le modifier, ou
					créez-en un nouveau.
				</p>
			</div>
		</div>
	)
}
