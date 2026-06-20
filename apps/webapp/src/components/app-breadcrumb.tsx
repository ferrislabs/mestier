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

interface BreadcrumbItemDefinition {
	id: string
	label: string
	to?: string
}

export function AppBreadcrumb() {
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const items = getBreadcrumbItems(location.pathname, activeOrganization.name)

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
): BreadcrumbItemDefinition[] {
	if (pathname === '/') {
		return [{ id: 'organization', label: organizationName }]
	}

	if (pathname.startsWith('/customers/')) {
		return [
			{ id: 'organization', label: organizationName, to: '/' },
			{ id: 'customers', label: 'Clients', to: '/customers' },
			{ id: 'customer-detail', label: 'Fiche client' },
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
