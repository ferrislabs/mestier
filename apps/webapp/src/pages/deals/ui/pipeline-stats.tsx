import { formatMoney } from '#/pages/customers/lib/format'
import {
	type PipelineSummary,
	SUMMARY_WINDOW_DAYS,
} from '#/pages/deals/lib/pipeline'

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

function countOf(count: number, one: string, many: string): string {
	return `${count} ${count > 1 ? many : one}`
}

export function PipelineStats({ summary }: { summary: PipelineSummary }) {
	return (
		<div className="mb-5 grid grid-cols-2 gap-3 lg:grid-cols-4">
			<Stat
				label="Pipeline ouvert"
				value={formatMoney(summary.openAmount)}
				hint={`HT, ${countOf(summary.openCount, 'opportunité ouverte', 'opportunités ouvertes')}`}
			/>
			<Stat
				label={`Gagné sur ${SUMMARY_WINDOW_DAYS} jours`}
				value={formatMoney(summary.wonAmount)}
				hint={`HT, ${countOf(summary.wonCount, 'opportunité gagnée', 'opportunités gagnées')}`}
			/>
			<Stat
				label="Taux de transformation"
				value={summary.winRate === null ? '—' : `${summary.winRate} %`}
				hint={`gagnées sur gagnées et perdues, ${SUMMARY_WINDOW_DAYS} jours`}
			/>
			<Stat
				label="Prospects devenus clients"
				value={String(summary.conversions)}
				hint={`sur les ${SUMMARY_WINDOW_DAYS} derniers jours`}
			/>
		</div>
	)
}
