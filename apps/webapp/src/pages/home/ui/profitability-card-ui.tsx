import { Link } from '@tanstack/react-router'
import { ArrowDownRight, ArrowRight, ArrowUpRight } from 'lucide-react'
import { buildOrgPath } from '#/modules/org-path'
import { formatCents } from '#/pages/reporting/types'

export interface ProfitabilityCardUIProps {
	periodLabel: string
	organizationSlug: string
	quotedCents: number
	marginCents: number
	/** #306: money fields are redacted server-side when the caller lacks
	 * `VIEW_COST` — mirrors `pages/reporting`'s own handling of
	 * `costs_redacted`, never re-decided here. */
	costsRedacted: boolean
	/**
	 * Change vs the previous full calendar month, in percent — `null` when it
	 * cannot be stated (still loading, an error, costs redacted, or the
	 * previous month had no complete project to compare against), in which
	 * case no trend is shown rather than a misleading "0%".
	 */
	trendPercent: number | null
	isLoading: boolean
	error: string | null
}

/**
 * Rentabilité as the homepage's lead card — a real figure and, when one can
 * be stated, how it moved since last month, with a link through to the full
 * `/reporting` page rather than a breakdown here. Sized and placed to read
 * first among the homepage's cards; a full dashboard still lives at
 * `/reporting`, not here (#391, #398).
 */
export function ProfitabilityCardUI({
	periodLabel,
	organizationSlug,
	quotedCents,
	marginCents,
	costsRedacted,
	trendPercent,
	isLoading,
	error,
}: ProfitabilityCardUIProps) {
	const showFigures = !error && !isLoading && !costsRedacted

	const value = error
		? '—'
		: isLoading
			? '…'
			: costsRedacted
				? '—'
				: formatCents(marginCents)

	const hint = error
		? error
		: isLoading
			? 'Chargement…'
			: costsRedacted
				? `Accès restreint · ${periodLabel}`
				: `Devisé ${formatCents(quotedCents)} · ${periodLabel}`

	return (
		<Link
			to={buildOrgPath(organizationSlug, '/reporting')}
			className="flex flex-col gap-3 rounded-xl bg-card p-6 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
		>
			<span className="island-kicker">Marge nette</span>

			<div className="flex flex-wrap items-baseline gap-3">
				<span className="text-4xl font-semibold tracking-tight text-foreground tabular-nums md:text-[2.75rem]">
					{value}
				</span>
				{showFigures && trendPercent !== null ? (
					<TrendBadge percent={trendPercent} />
				) : null}
			</div>

			<p className="text-sm text-muted-foreground">{hint}</p>

			<span className="mt-1 inline-flex items-center gap-1.5 text-sm font-medium text-brand-muted">
				Voir le rapport complet
				<ArrowRight className="size-3.5" />
			</span>
		</Link>
	)
}

function TrendBadge({ percent }: { percent: number }) {
	const isPositive = percent >= 0
	const Icon = isPositive ? ArrowUpRight : ArrowDownRight

	return (
		<span
			className={
				isPositive
					? 'inline-flex items-center gap-1 text-sm font-semibold text-success'
					: 'inline-flex items-center gap-1 text-sm font-semibold text-destructive'
			}
		>
			<Icon className="size-3.5" />
			{isPositive ? '+' : ''}
			{percent.toFixed(0)}&nbsp;% vs mois dernier
		</span>
	)
}
