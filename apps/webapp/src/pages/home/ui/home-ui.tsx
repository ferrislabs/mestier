import { Link } from '@tanstack/react-router'
import { Receipt, Users } from 'lucide-react'
import type * as React from 'react'
import { MetricCard, PageShell } from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import type { SearchGroup } from '#/pages/home/ui/home-search-ui'
import { HomeSearchUI } from '#/pages/home/ui/home-search-ui'
import { ProfitabilityCardUI } from '#/pages/home/ui/profitability-card-ui'

interface HomeStats {
	customers: number
	invoices: number
	/** Summed from `useOutstandingByCustomer`, `null` while that read is in
	 * flight or has failed — never fabricated as 0 in the meantime. */
	outstandingCents: number | null
}

interface HomeProfitability {
	periodLabel: string
	organizationSlug: string
	quotedCents: number
	marginCents: number
	costsRedacted: boolean
	isLoading: boolean
	error: string | null
}

const QUICK_LINKS = [
	{ label: 'Clients', to: '/crm/customers' },
	{ label: 'Devis', to: '/crm/quotes' },
	{ label: 'Planning', to: '/planning' },
	{ label: 'Achats', to: '/purchase' },
] as const

interface HomeUIProps {
	userName?: string
	stats: HomeStats
	profitability: HomeProfitability
	organizationSlug: string
	search: SearchGroup[]
	/** The self-service pointage card, composed in by the feature layer. */
	todayTasks?: React.ReactNode
}

export function HomeUI({
	userName,
	stats,
	profitability,
	organizationSlug,
	search,
	todayTasks,
}: HomeUIProps) {
	return (
		<PageShell>
			<div className="mx-auto flex w-full max-w-2xl flex-col items-center gap-6 pt-6 pb-2 text-center">
				<h1 className="text-2xl font-normal text-foreground">
					{userName ? `Bonjour, ${userName}` : 'Bonjour'}
				</h1>

				<HomeSearchUI groups={search} />

				<nav className="flex flex-wrap items-center justify-center gap-2">
					{QUICK_LINKS.map((link) => (
						<Link
							key={link.to}
							to={buildOrgPath(organizationSlug, link.to)}
							className="rounded-full border px-4 py-1.5 text-sm text-muted-foreground transition hover:border-primary/40 hover:bg-muted hover:text-foreground"
						>
							{link.label}
						</Link>
					))}
				</nav>
			</div>

			<section className="grid grid-cols-1 gap-4 sm:grid-cols-3">
				<ProfitabilityCardUI
					periodLabel={profitability.periodLabel}
					organizationSlug={profitability.organizationSlug}
					quotedCents={profitability.quotedCents}
					marginCents={profitability.marginCents}
					costsRedacted={profitability.costsRedacted}
					isLoading={profitability.isLoading}
					error={profitability.error}
				/>
				<MetricCard
					label="Clients"
					value={stats.customers.toString()}
					hint="Total enregistrés"
					icon={<Users className="size-4" />}
				/>
				<MetricCard
					label="Factures"
					value={stats.invoices.toString()}
					hint={
						stats.outstandingCents === null
							? 'Total émises'
							: `Total émises · ${formatOutstandingHint(stats.outstandingCents)}`
					}
					icon={<Receipt className="size-4" />}
				/>
			</section>

			{todayTasks}
		</PageShell>
	)
}

function formatOutstandingHint(outstandingCents: number): string {
	if (outstandingCents <= 0) return 'rien en attente'

	const amount = new Intl.NumberFormat('fr-FR', {
		style: 'currency',
		currency: 'EUR',
	}).format(outstandingCents / 100)

	return `${amount} en attente`
}
