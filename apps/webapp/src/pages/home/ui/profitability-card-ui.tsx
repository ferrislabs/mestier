import { Link } from '@tanstack/react-router'
import { TrendingUp } from 'lucide-react'
import { MetricCard } from '#/components/ui/surface'
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
	isLoading: boolean
	error: string | null
}

/**
 * Rentabilité as one secondary tile among peers, not the page's centerpiece —
 * a glance at the margin with a link through to the full `/reporting` page,
 * not a full breakdown (that used to live here as a hero section; see #391).
 */
export function ProfitabilityCardUI({
	periodLabel,
	organizationSlug,
	quotedCents,
	marginCents,
	costsRedacted,
	isLoading,
	error,
}: ProfitabilityCardUIProps) {
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
			className="block rounded-xl transition hover:-translate-y-0.5"
		>
			<MetricCard
				label="Marge (rentabilité)"
				value={value}
				hint={hint}
				icon={<TrendingUp className="size-4" />}
			/>
		</Link>
	)
}
