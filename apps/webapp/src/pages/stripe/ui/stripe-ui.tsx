import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell } from '#/components/ui/surface'
import type { Customer } from '#/hooks/use-customers'
import { Notice } from '#/pages/customers/ui/notice'
import { StatTile } from '#/pages/customers/ui/stat-tile'
import {
	hasStripeFilters,
	STRIPE_STATS_WINDOW_DAYS,
	type StripeSearch,
	type StripeSearchPatch,
	type StripeStats,
} from '#/pages/stripe/lib/stripe-model'
import type { StripeEvent, StripeScenario } from '#/pages/stripe/types'
import { HistoryTable } from '#/pages/stripe/ui/history-table'
import { ReviewCard } from '#/pages/stripe/ui/review-card'
import { SimulateMenu } from '#/pages/stripe/ui/simulate-menu'
import { StripeToolbar } from '#/pages/stripe/ui/stripe-toolbar'
import { WebhookPanel } from '#/pages/stripe/ui/webhook-panel'
import type { OpenInvoice } from '#/pages/treasury/types'

export function StripeUI({
	organizationSlug,
	endpoint,
	secretSet,
	search,
	onSearchChange,
	notice,
	onNotice,
	onDismissNotice,
	stats,
	toReview,
	history,
	customers,
	openInvoices,
	onSimulate,
	onLink,
	onAllocate,
	onIgnore,
}: {
	organizationSlug: string
	endpoint: string
	secretSet: boolean
	search: StripeSearch
	onSearchChange: (patch: StripeSearchPatch) => void
	notice: string | null
	onNotice: (message: string) => void
	onDismissNotice: () => void
	stats: StripeStats
	toReview: readonly StripeEvent[]
	history: readonly StripeEvent[]
	customers: readonly Customer[]
	openInvoices: readonly OpenInvoice[]
	onSimulate: (scenario: StripeScenario) => void
	onLink: (event: StripeEvent, customer: Customer) => void
	onAllocate: (event: StripeEvent, invoice: OpenInvoice) => void
	onIgnore: (event: StripeEvent) => void
}) {
	const windowLabel = `${STRIPE_STATS_WINDOW_DAYS} derniers jours`
	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6 flex-row flex-wrap items-end justify-between gap-4"
				title="Stripe"
				description="Les paiements reçus par Stripe, rapprochés de tes factures et de tes clients."
				actions={<SimulateMenu onSimulate={onSimulate} />}
			/>
			<Notice message={notice} onDismiss={onDismissNotice} />
			<WebhookPanel
				organizationSlug={organizationSlug}
				endpoint={endpoint}
				secretSet={secretSet}
				onNotice={onNotice}
			/>
			<section
				aria-label="Chiffres clés Stripe"
				className="mb-6 grid grid-cols-2 gap-3 lg:grid-cols-4"
			>
				<StatTile
					label="Événements reçus"
					value={stats.received}
					hint={windowLabel}
				/>
				<StatTile
					label="À rapprocher"
					value={stats.toReview}
					tone={stats.toReview > 0 ? 'alert' : 'ok'}
					hint="Toutes périodes"
				/>
				<StatTile
					label="Traités automatiquement"
					value={stats.automatic}
					hint={windowLabel}
				/>
				<StatTile label="Ignorés" value={stats.ignored} hint={windowLabel} />
			</section>
			<section aria-labelledby="stripe-review-title" className="mb-8">
				<h2 id="stripe-review-title" className="mb-3 font-semibold text-lg">
					À rapprocher
					{toReview.length > 0 && (
						<span className="ml-2 font-normal text-muted-foreground text-sm">
							{toReview.length} événement{toReview.length > 1 ? 's' : ''}
						</span>
					)}
				</h2>
				{toReview.length === 0 ? (
					<EmptyState
						title="Tout est rapproché"
						hint="Aucun paiement Stripe n’attend ton attention. Les prochains événements douteux apparaîtront ici."
					/>
				) : (
					<ul className="grid gap-3">
						{toReview.map((event) => (
							<ReviewCard
								key={event.id}
								event={event}
								organizationSlug={organizationSlug}
								customers={customers}
								openInvoices={openInvoices}
								onLink={onLink}
								onAllocate={onAllocate}
								onIgnore={onIgnore}
							/>
						))}
					</ul>
				)}
			</section>
			<section aria-labelledby="stripe-history-title">
				<h2 id="stripe-history-title" className="mb-3 font-semibold text-lg">
					Historique
				</h2>
				<StripeToolbar search={search} onChange={onSearchChange} />
				{history.length === 0 ? (
					<EmptyState
						title={
							hasStripeFilters(search)
								? 'Aucun événement ne correspond à ces filtres'
								: 'Aucun événement reçu pour le moment'
						}
						hint={
							hasStripeFilters(search)
								? 'Retire un filtre pour retrouver les événements.'
								: 'Simule un webhook pour voir le rapprochement à l’œuvre.'
						}
					>
						{hasStripeFilters(search) && (
							<Button
								variant="outline"
								onClick={() =>
									onSearchChange({ statut: null, type: null, q: null })
								}
							>
								Réinitialiser les filtres
							</Button>
						)}
					</EmptyState>
				) : (
					<HistoryTable events={history} organizationSlug={organizationSlug} />
				)}
			</section>
		</PageShell>
	)
}
