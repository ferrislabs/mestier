import { Link, useLocation } from '@tanstack/react-router'
import { Fragment } from 'react'
import {
	Breadcrumb,
	BreadcrumbItem,
	BreadcrumbLink,
	BreadcrumbList,
	BreadcrumbPage,
	BreadcrumbSeparator,
} from '#/components/ui/breadcrumb'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomer } from '#/hooks/use-customers'
import { customerDisplayName } from '#/pages/customers/types'

interface BreadcrumbItemDefinition {
	id: string
	label: string
	to?: string
}

export function AppBreadcrumb() {
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const customerId = getCustomerId(location.pathname)
	const customer = useCustomer(customerId ?? '', Boolean(customerId))
	const customerLabel = customer.data?.data
		? customerDisplayName(customer.data.data)
		: 'Fiche client'
	const items = getBreadcrumbItems(
		location.pathname,
		activeOrganization.name,
		customerLabel,
	)

	return (
		<Breadcrumb className="min-w-0 flex-1">
			<BreadcrumbList className="flex-nowrap overflow-hidden">
				{items.map((item, index) => {
					const isLast = index === items.length - 1

					return (
						<Fragment key={item.id}>
							<BreadcrumbItem className="min-w-0">
								{isLast || !item.to ? (
									<BreadcrumbPage className="truncate font-medium">
										{item.label}
									</BreadcrumbPage>
								) : (
									<BreadcrumbLink asChild className="truncate">
										<Link to={item.to}>{item.label}</Link>
									</BreadcrumbLink>
								)}
							</BreadcrumbItem>
							{isLast ? null : <BreadcrumbSeparator />}
						</Fragment>
					)
				})}
			</BreadcrumbList>
		</Breadcrumb>
	)
}

function getBreadcrumbItems(
	pathname: string,
	organizationName: string,
	customerLabel: string,
): BreadcrumbItemDefinition[] {
	if (pathname === '/') {
		return [{ id: 'organization', label: organizationName }]
	}

	if (pathname.startsWith('/customers/')) {
		return [
			{ id: 'organization', label: organizationName, to: '/' },
			{ id: 'customers', label: 'Clients', to: '/customers' },
			{ id: 'customer-detail', label: customerLabel },
		]
	}

	if (pathname.startsWith('/customers')) {
		return [
			{ id: 'organization', label: organizationName, to: '/' },
			{ id: 'customers', label: 'Clients' },
		]
	}

	if (pathname.startsWith('/quotes')) {
		return [
			{ id: 'organization', label: organizationName, to: '/' },
			{ id: 'quotes', label: 'Devis' },
		]
	}

	if (pathname.startsWith('/settings')) {
		return [
			{ id: 'organization', label: organizationName, to: '/' },
			{ id: 'settings', label: 'Paramètres' },
		]
	}

	return [
		{ id: 'organization', label: organizationName, to: '/' },
		{ id: 'console', label: 'Console' },
	]
}

function getCustomerId(pathname: string): string | null {
	const match = /^\/customers\/([^/]+)$/.exec(pathname)
	if (!match?.[1]) return null
	return decodeURIComponent(match[1])
}
