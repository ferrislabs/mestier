import { Receipt, Users } from 'lucide-react'
import type * as React from 'react'
import { MetricCard, PageShell } from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import { MODULES } from '#/modules/registry'
import { AppLauncherUI } from '#/pages/home/ui/app-launcher-ui'
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
	trendPercent: number | null
	isLoading: boolean
	error: string | null
}

interface HomeUIProps {
	userName?: string
	stats: HomeStats
	profitability: HomeProfitability
	organizationSlug: string
	search: SearchGroup[]
	/** The self-service pointage card, composed in by the feature layer. */
	todayTasks?: React.ReactNode
	/** Today's team-wide agenda, composed in by the feature layer. */
	todayPlanning: React.ReactNode
	/** A glance at recent chat activity, composed in by the feature layer. */
	discussions: React.ReactNode
}

export function HomeUI({
	userName,
	stats,
	profitability,
	organizationSlug,
	search,
	todayTasks,
	todayPlanning,
	discussions,
}: HomeUIProps) {
	// Every module but Accueil itself — a shortcut back to the page you're
	// already on would be dead weight in a launcher grid.
	const launcherItems = MODULES.filter((module) => module.id !== 'home').map(
		(module) => ({
			id: module.id,
			label: module.label,
			icon: module.icon,
			to: buildOrgPath(organizationSlug, module.basePath),
		}),
	)

	return (
		<PageShell>
			<div className="mx-auto flex w-full max-w-2xl flex-col items-center gap-6 pt-6 pb-2 text-center">
				<h1 className="text-2xl font-normal text-foreground">
					{userName ? `Bonjour, ${userName}` : 'Bonjour'}
				</h1>

				<HomeSearchUI groups={search} />

				<AppLauncherUI items={launcherItems} />
			</div>

			<section className="grid grid-cols-1 gap-4 lg:grid-cols-[1.7fr_1fr]">
				<ProfitabilityCardUI
					periodLabel={profitability.periodLabel}
					organizationSlug={profitability.organizationSlug}
					quotedCents={profitability.quotedCents}
					marginCents={profitability.marginCents}
					costsRedacted={profitability.costsRedacted}
					trendPercent={profitability.trendPercent}
					isLoading={profitability.isLoading}
					error={profitability.error}
				/>
				<div className="grid grid-cols-2 gap-4 lg:grid-cols-1">
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
				</div>
			</section>

			<section className="grid grid-cols-1 gap-4 md:grid-cols-2">
				{todayPlanning}
				{discussions}
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
