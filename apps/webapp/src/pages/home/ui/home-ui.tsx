import { Receipt, Users } from 'lucide-react'
import type * as React from 'react'
import { MetricCard, PageShell } from '#/components/ui/surface'
import { buildOrgPath } from '#/modules/org-path'
import type { AppModule } from '#/modules/types'
import {
	activityCountLabel,
	formatToday,
	greeting,
	type HomeActivity,
} from '#/pages/home/types'
import { ActivityFeedUI } from '#/pages/home/ui/activity-feed-ui'
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
	organizationName: string
	stats: HomeStats
	profitability: HomeProfitability
	organizationSlug: string
	search: SearchGroup[]
	activity: HomeActivity[]
	isActivityLoading: boolean
	/** The self-service pointage card, composed in by the feature layer. */
	todayTasks?: React.ReactNode
	/** Today's team-wide agenda, composed in by the feature layer. */
	todayPlanning: React.ReactNode
	/** A glance at recent chat activity, composed in by the feature layer. */
	discussions: React.ReactNode
	modules: AppModule[]
	today?: Date
}

export function HomeUI({
	userName,
	organizationName,
	stats,
	profitability,
	organizationSlug,
	search,
	activity,
	isActivityLoading,
	todayTasks,
	todayPlanning,
	discussions,
	modules,
	today = new Date(),
}: HomeUIProps) {
	// Every module but Accueil itself — a shortcut back to the page you're
	// already on would be dead weight in a launcher grid.
	const launcherItems = modules
		.filter((module) => module.id !== 'home')
		.map((module) => ({
			id: module.id,
			label: module.label,
			module,
			to: buildOrgPath(organizationSlug, module.basePath),
			hint: activityCountLabel(
				activity.filter((item) => item.module.id === module.id).length,
			),
			comingSoon: module.status !== 'available',
		}))

	return (
		<PageShell className="gap-0">
			<header className="flex flex-wrap items-end justify-between gap-2">
				<div>
					<p className="text-[13px] text-muted-foreground">
						{formatToday(today)}, {organizationName}
					</p>
					<h1 className="mt-0.5 font-semibold text-2xl tracking-tight">
						{greeting(today)}
						{userName ? ` ${userName}` : ''}
					</h1>
				</div>
			</header>

			<div className="mt-6">
				<HomeSearchUI groups={search} />
			</div>

			<AppLauncherUI items={launcherItems} />

			<section
				aria-label="Indicateurs"
				className="mt-10 grid grid-cols-1 gap-4 lg:grid-cols-[1.7fr_1fr]"
			>
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

			<div className="mt-6">
				<ActivityFeedUI activity={activity} isLoading={isActivityLoading} />
			</div>

			<section className="mt-6 grid grid-cols-1 items-start gap-4 md:grid-cols-2">
				{todayPlanning}
				{discussions}
			</section>

			{todayTasks ? <div className="mt-6">{todayTasks}</div> : null}
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
