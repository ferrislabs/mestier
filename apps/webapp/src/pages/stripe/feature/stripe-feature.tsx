import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useCrmSettings } from '#/pages/crm-settings/feature/mocks/settings'
import { useSaveAccountExtras } from '#/pages/customers/feature/mocks/account-extras'
import { useNotice } from '#/pages/customers/feature/use-notice'
import { todayIso } from '#/pages/customers/lib/format'
import {
	useAllocateStripeEvent,
	useIgnoreStripeEvent,
	useLinkStripeCustomer,
	useSimulateStripeEvent,
	useStripeEvents,
} from '#/pages/stripe/feature/mocks/stripe-events'
import {
	filterStripeEvents,
	linkNotice,
	type StripeSearch,
	type StripeSearchPatch,
	simulationNotice,
	stripeStats,
	stripeToReview,
} from '#/pages/stripe/lib/stripe-model'
import { StripeUI } from '#/pages/stripe/ui/stripe-ui'
import { useLedger } from '#/pages/treasury/feature/mocks/ledger'

interface StripeFeatureProps {
	search: StripeSearch
	onSearchChange: (patch: StripeSearchPatch) => void
}

export function StripeFeature({ search, onSearchChange }: StripeFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const notice = useNotice()
	const events = useStripeEvents().data
	const settings = useCrmSettings().data.settings
	const ledger = useLedger().data
	const customers = useCustomers(activeOrganization.id, {
		page: 1,
		perPage: 100,
	})
	const link = useLinkStripeCustomer()
	const allocate = useAllocateStripeEvent()
	const ignore = useIgnoreStripeEvent()
	const simulate = useSimulateStripeEvent()
	const saveExtras = useSaveAccountExtras()

	return (
		<StripeUI
			organizationSlug={activeOrganization.slug}
			endpoint={settings.stripeEndpoint}
			secretSet={settings.stripeSecretSet}
			search={search}
			onSearchChange={onSearchChange}
			notice={notice.message}
			onNotice={notice.show}
			onDismissNotice={notice.clear}
			stats={stripeStats(events, todayIso())}
			toReview={stripeToReview(events)}
			history={filterStripeEvents(events, search)}
			customers={(customers.data?.data ?? []).filter(
				(customer) => customer.status !== 'ARCHIVED',
			)}
			openInvoices={ledger.openInvoices}
			onSimulate={(scenario) => {
				void simulate
					.mutateAsync(scenario, ledger.openInvoices[0]?.label ?? null)
					.then((created) => notice.show(simulationNotice(scenario, created)))
			}}
			onLink={(event, customer) => {
				void link.mutateAsync(event.id, customer).then(() => {
					if (event.customerId)
						void saveExtras.mutateAsync(customer.id, {
							stripeCustomerId: event.customerId,
						})
					notice.show(linkNotice(customer.name))
				})
			}}
			onAllocate={(event, invoice) => {
				void allocate
					.mutateAsync(event.id, invoice.label)
					.then(() => notice.show(`Paiement affecté à ${invoice.label}.`))
			}}
			onIgnore={(event) => {
				void ignore
					.mutateAsync(event.id)
					.then(() => notice.show('Événement ignoré.'))
			}}
		/>
	)
}
