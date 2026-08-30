import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useInvoices, useOutstandingByCustomer } from '#/hooks/use-invoices'
import { useProjects } from '#/hooks/use-projects'
import { useQuotes } from '#/hooks/use-quotes'
import { type Period, useProfitability } from '#/hooks/use-reporting'
import { buildOrgPath } from '#/modules/org-path'
import { DiscussionsFeature } from '#/pages/home/feature/discussions-feature'
import { MyTasksTodayFeature } from '#/pages/home/feature/my-tasks-today-feature'
import { TodayPlanningFeature } from '#/pages/home/feature/today-planning-feature'
import type { SearchGroup } from '#/pages/home/ui/home-search-ui'
import { HomeUI } from '#/pages/home/ui/home-ui'
import {
	currentMonthPeriod,
	isCompleteProject,
	previousMonthPeriod,
} from '#/pages/reporting/types'

export function HomeFeature() {
	const { activeOrganizationId, activeOrganization } = useActiveOrganization()

	// Fixed to the current month: this is a glance, not the reporting page's own
	// date range picker. Anyone wanting another period follows the CTA to
	// `/reporting`, which owns that control.
	const [period] = useState<Period>(() => currentMonthPeriod(new Date()))
	// The full previous month, read once alongside it purely for the trend
	// figure ("+8% vs juillet") — never shown itself, never a control.
	const [previousPeriod] = useState<Period>(() =>
		previousMonthPeriod(new Date()),
	)

	const profitability = useProfitability(activeOrganizationId, period)
	const previousProfitability = useProfitability(
		activeOrganizationId,
		previousPeriod,
	)
	// Default page size, same as every other caller of these hooks — so the
	// query key matches whatever `/crm/customers`, `/crm/quotes`, `/planning`
	// or `/invoices` already cached, rather than forcing a fresh fetch just for
	// a homepage counter or search entry.
	const customers = useCustomers(activeOrganizationId)
	const invoices = useInvoices(activeOrganizationId)
	const outstanding = useOutstandingByCustomer(activeOrganizationId)
	const quotes = useQuotes(activeOrganizationId)
	const projects = useProjects(activeOrganizationId, {
		includeArchived: false,
	})

	const report = profitability.data?.data
	const completeProjects = (report?.projects ?? []).filter(isCompleteProject)

	const quotedCents = completeProjects.reduce(
		(sum, project) => sum + (project.quoted_cents ?? 0),
		0,
	)
	const marginCents = completeProjects.reduce(
		(sum, project) => sum + (project.margin_cents ?? 0),
		0,
	)

	const previousReport = previousProfitability.data?.data
	const previousCompleteProjects = (previousReport?.projects ?? []).filter(
		isCompleteProject,
	)
	const previousMarginCents = previousCompleteProjects.reduce(
		(sum, project) => sum + (project.margin_cents ?? 0),
		0,
	)

	// `null` — no trend shown — whenever either period isn't in a state where
	// the comparison means something: still loading, an error, costs redacted
	// on either side, or a non-positive previous margin (a percentage change
	// off zero or a loss is not a sentence worth showing).
	const trendPercent =
		!profitability.isLoading &&
		!previousProfitability.isLoading &&
		!profitability.error &&
		!previousProfitability.error &&
		report?.costs_redacted === false &&
		previousReport?.costs_redacted === false &&
		previousMarginCents > 0
			? ((marginCents - previousMarginCents) / previousMarginCents) * 100
			: null

	// One number per customer, already computed server-side — summed here for
	// the tile, not recomputed. Same pattern as the invoice list's own
	// outstanding total (`invoice-list-feature.tsx`).
	const outstandingCents = outstanding.data
		? outstanding.data.data.reduce(
				(sum, balance) => sum + balance.outstanding_cents,
				0,
			)
		: null

	// No search endpoint exists yet: this filters over whatever's already
	// loaded for the stat tiles (plus quotes/projects, fetched here for that
	// purpose alone). Fine at artisan/SME scale; revisit once an organization
	// outgrows a single page of any of these lists.
	const searchGroups: SearchGroup[] = [
		{
			label: 'Clients',
			items: (customers.data?.data ?? []).map((customer) => ({
				id: customer.id,
				label: customer.name,
				to: buildOrgPath(
					activeOrganization.slug,
					`/crm/customers/${customer.id}`,
				),
			})),
		},
		{
			label: 'Devis',
			items: (quotes.data?.data ?? []).map((quote) => ({
				id: quote.id,
				label: quote.title,
				sublabel: quote.reference ?? undefined,
				to: buildOrgPath(activeOrganization.slug, `/crm/quotes/${quote.id}`),
			})),
		},
		{
			label: 'Projets',
			items: (projects.data?.data ?? []).map((project) => ({
				id: project.id,
				label: project.name,
				to: buildOrgPath(
					activeOrganization.slug,
					`/planning/projects/${project.id}`,
				),
			})),
		},
		{
			label: 'Factures',
			items: (invoices.data?.data ?? []).map((invoice) => ({
				id: invoice.id,
				label: invoice.number ?? 'Facture sans numéro',
				to: buildOrgPath(
					activeOrganization.slug,
					`/crm/invoices/${invoice.id}`,
				),
			})),
		},
	]

	return (
		<HomeUI
			userName="Nathael"
			organizationSlug={activeOrganization.slug}
			search={searchGroups}
			stats={{
				customers:
					customers.data?.pagination?.total ?? customers.data?.data.length ?? 0,
				invoices:
					invoices.data?.pagination?.total ?? invoices.data?.data.length ?? 0,
				outstandingCents,
			}}
			profitability={{
				periodLabel: monthLabel(period),
				organizationSlug: activeOrganization.slug,
				quotedCents,
				marginCents,
				costsRedacted: report?.costs_redacted ?? true,
				trendPercent,
				isLoading: profitability.isLoading,
				error: profitability.error?.message ?? null,
			}}
			todayTasks={<MyTasksTodayFeature organizationId={activeOrganizationId} />}
			todayPlanning={
				<TodayPlanningFeature
					organizationId={activeOrganizationId}
					organizationSlug={activeOrganization.slug}
				/>
			}
			discussions={
				<DiscussionsFeature
					organizationId={activeOrganizationId}
					organizationSlug={activeOrganization.slug}
				/>
			}
		/>
	)
}

/** `août 2026`, parsed from the ISO date directly rather than through `new
 * Date(period.from)` — that constructor reads the string as UTC midnight,
 * which a browser west of Greenwich rolls back to the previous day. */
function monthLabel(period: Period): string {
	const [year, month] = period.from.split('-').map(Number)
	return new Intl.DateTimeFormat('fr-FR', {
		month: 'long',
		year: 'numeric',
	}).format(new Date(year, month - 1, 1))
}
