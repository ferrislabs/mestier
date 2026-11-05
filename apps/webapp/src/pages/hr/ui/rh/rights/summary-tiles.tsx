import { cn } from '#/lib/utils'
import {
	formatAmount,
	LOW_BALANCE_THRESHOLD,
	type RightsTotals,
} from '#/pages/hr/lib/rh/rights-model'
import type { Member } from '#/pages/hr/lib/rh/types'

function Tile({
	label,
	value,
	unit,
	hint,
	tone,
}: {
	label: string
	value: string
	unit?: string
	hint: string
	tone?: string
}) {
	return (
		<article className="rounded-lg border bg-card px-4 py-3.5">
			<h3 className="text-[13px] text-muted-foreground">{label}</h3>
			<p className={cn('tabular mt-1 font-semibold text-2xl', tone)}>
				{value}
				{unit && (
					<span className="ml-1 font-normal text-muted-foreground text-sm">
						{unit}
					</span>
				)}
			</p>
			<p className="mt-1 text-[13px] text-muted-foreground">{hint}</p>
		</article>
	)
}

function listNames(names: readonly string[]): string {
	if (names.length <= 1) return names.join('')
	return `${names.slice(0, -1).join(', ')} et ${names[names.length - 1]}`
}

export function SummaryTiles({
	totals,
	lowMembers,
}: {
	totals: RightsTotals
	lowMembers: readonly Member[]
}) {
	const members = totals.count > 1 ? 'membres' : 'membre'
	return (
		<section
			aria-label="Synthèse de l’équipe"
			className="grid gap-3 sm:grid-cols-3"
		>
			<Tile
				label="Solde cumulé de l’équipe"
				value={formatAmount(totals.balance)}
				unit="j"
				hint={`${totals.count} ${members} actifs sur la période`}
				tone={totals.balance < 0 ? 'text-state-alert' : undefined}
			/>
			<Tile
				label="Jours posés à venir"
				value={formatAmount(totals.planned)}
				unit="j"
				hint={
					totals.pending > 0
						? `et ${formatAmount(totals.pending)} j en attente de validation`
						: 'Aucune demande en attente'
				}
			/>
			<Tile
				label="Soldes faibles"
				value={String(lowMembers.length)}
				hint={
					lowMembers.length > 0
						? listNames(lowMembers.map((member) => member.firstName))
						: `Aucun disponible sous ${LOW_BALANCE_THRESHOLD} jours`
				}
				tone={lowMembers.length > 0 ? 'text-state-today' : undefined}
			/>
		</section>
	)
}
