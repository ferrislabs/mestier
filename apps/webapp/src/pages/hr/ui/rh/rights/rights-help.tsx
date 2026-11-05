import { ChevronRight } from 'lucide-react'

const LINES = [
	{
		term: 'Solde',
		text: 'acquis + report + régularisations − jours pris (absences validées jusqu’à aujourd’hui).',
	},
	{
		term: 'Posés',
		text: 'absences validées à venir. En attente : demandes pas encore validées.',
	},
	{
		term: 'Disponible',
		text: 'solde − posés − en attente, soit ce que le membre peut encore demander.',
	},
	{
		term: 'Fin de période',
		text: 'acquisition complète de la période − tout ce qui est pris, posé ou en attente.',
	},
]

export function RightsHelp() {
	return (
		<details className="group rounded-lg border bg-card px-4 py-3 text-sm">
			<summary className="flex cursor-pointer list-none items-center gap-2 font-medium [&::-webkit-details-marker]:hidden">
				<ChevronRight className="size-4 text-muted-foreground transition-transform group-open:rotate-90" />
				Comment sont calculés les soldes ?
			</summary>
			<dl className="mt-3 grid gap-1.5 pl-6 text-muted-foreground">
				{LINES.map((line) => (
					<div key={line.term}>
						<dt className="inline font-medium text-foreground">{line.term}</dt>
						<dd className="inline"> : {line.text}</dd>
					</div>
				))}
			</dl>
		</details>
	)
}
