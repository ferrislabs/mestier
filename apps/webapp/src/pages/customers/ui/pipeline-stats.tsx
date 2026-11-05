function Stat({
	label,
	value,
	hint,
}: {
	label: string
	value: string
	hint: string
}) {
	return (
		<div className="rounded-lg border border-border bg-card px-4 py-3">
			<p className="text-muted-foreground text-sm">{label}</p>
			<p className="mt-1 font-semibold text-2xl tabular-nums">{value}</p>
			<p className="mt-0.5 text-muted-foreground text-xs">{hint}</p>
		</div>
	)
}

export interface PipelineSummary {
	total: number
	active: number
	quotes: number
	won: number
}

export function PipelineStats({ summary }: { summary: PipelineSummary }) {
	return (
		<div className="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
			<Stat
				label="Opportunités"
				value={String(summary.total)}
				hint="Prospects et clients actifs"
			/>
			<Stat
				label="En cours"
				value={String(summary.active)}
				hint="Hors gagné/perdu"
			/>
			<Stat
				label="Devis envoyés"
				value={String(summary.quotes)}
				hint="À suivre"
			/>
			<Stat
				label="Gagnés"
				value={String(summary.won)}
				hint="Convertis clients"
			/>
		</div>
	)
}
