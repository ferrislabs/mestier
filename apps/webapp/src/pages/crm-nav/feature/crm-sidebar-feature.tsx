import { useLocation } from '@tanstack/react-router'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomers } from '#/hooks/use-customers'
import { useInvoices } from '#/hooks/use-invoices'
import { type PermissionName, usePermissions } from '#/hooks/use-permissions'
import { useQuotes } from '#/hooks/use-quotes'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { useSuppliers } from '#/hooks/use-suppliers'
import { splitOrgPath } from '#/modules/org-path'
import { activeSidebarEntries } from '#/pages/crm-nav/lib/crm-path'
import { CrmSidebar } from '#/pages/crm-nav/ui/crm-sidebar'
import { invoiceDisplayState } from '#/pages/invoices/types'
import { useSidebarGroups } from '#/pages/projet/feature/use-sidebar-groups'
import { useStripeEvents } from '#/pages/stripe/feature/mocks/stripe-events'

const COLLAPSED_KEY = 'mestier:crm:sidebar-collapsed'

export function CrmSidebarFeature() {
	const { activeOrganization } = useActiveOrganization()
	const [isCollapsed, toggleGroup] = useSidebarGroups(COLLAPSED_KEY)
	const location = useLocation({
		select: (current) => ({
			pathname: current.pathname,
			search: current.search as Record<string, unknown>,
		}),
	})
	const modulePath = splitOrgPath(location.pathname).path

	const permissions = usePermissions()
	const customers = useCustomers(activeOrganization.id)
	const quotes = useQuotes(activeOrganization.id)
	const invoices = useInvoices(activeOrganization.id)
	const catalog = useReferenceCatalog(activeOrganization.id, {
		members: false,
		employeeProfiles: false,
		equipment: false,
		customUnits: false,
	})
	const suppliers = useSuppliers(activeOrganization.id)
	const stripeEvents = useStripeEvents()

	const granted: readonly string[] = permissions.data?.data.permissions ?? []
	const visible = (permission: PermissionName | null) =>
		permission === null || granted.includes(permission)

	const invoiceRows = invoices.data?.data ?? []
	const now = new Date()

	return (
		<CrmSidebar
			organizationSlug={activeOrganization.slug}
			active={activeSidebarEntries(modulePath, location.search)}
			counts={{
				clients: customers.data
					? customers.data.data.filter(
							(customer) => customer.status !== 'ARCHIVED',
						).length
					: undefined,
				quotesAwaiting: quotes.data
					? quotes.data.data.filter((quote) => quote.status === 'SENT').length
					: undefined,
				invoicesOverdue: invoices.data
					? invoiceRows.filter(
							(invoice) =>
								invoice.kind !== 'CREDIT_NOTE' &&
								invoiceDisplayState(invoice, null, now) === 'overdue',
						).length
					: undefined,
				creditNotes: invoices.data
					? invoiceRows.filter((invoice) => invoice.kind === 'CREDIT_NOTE')
							.length
					: undefined,
				stripeToReview: stripeEvents.data.filter(
					(item) => item.status === 'to_review',
				).length,
				catalog:
					catalog.products.data && catalog.serviceRates.data
						? catalog.products.data.data.length +
							catalog.serviceRates.data.data.length
						: undefined,
				suppliers: suppliers.data?.data.length,
			}}
			visible={visible}
			isCollapsed={isCollapsed}
			onToggleGroup={toggleGroup}
		/>
	)
}
