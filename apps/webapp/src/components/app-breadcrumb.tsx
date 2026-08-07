import { useQuery } from '@tanstack/react-query'
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
import { buildBreadcrumbItems } from '#/modules/breadcrumb'
import { customerDisplayName } from '#/pages/customers/types'

export function AppBreadcrumb() {
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	const customerId = getCustomerId(location.pathname)
	const customer = useCustomer(customerId ?? '', Boolean(customerId))
	const customerLabel = customer.data?.data
		? customerDisplayName(customer.data.data)
		: 'Fiche client'

	const employeeId = getEmployeeWorkTimeId(location.pathname)
	const employee = useQuery({
		...window.tanstackApi.get('/api/v1/employees/{employee_id}', {
			path: { employee_id: employeeId ?? '' },
		}).queryOptions,
		enabled: Boolean(employeeId),
	})
	const employeeLabel = employee.data?.data?.name ?? 'Temps de travail'

	const items = buildBreadcrumbItems({
		pathname: location.pathname,
		organizationName: activeOrganization.name,
		detailLabel: getDetailLabel(
			location.pathname,
			customerLabel,
			employeeLabel,
		),
	})

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

function getDetailLabel(
	pathname: string,
	customerLabel: string,
	employeeLabel: string,
): string | undefined {
	if (getCustomerId(pathname)) return customerLabel
	if (/^\/crm\/quotes\/[^/]+$/.test(pathname)) return 'Fiche devis'
	if (getEmployeeWorkTimeId(pathname)) return employeeLabel
	return undefined
}

function getCustomerId(pathname: string): string | null {
	if (pathname === '/crm/customers/pipeline') return null
	const match = /^\/crm\/customers\/([^/]+)$/.exec(pathname)
	if (!match?.[1]) return null
	return decodeURIComponent(match[1])
}

function getEmployeeWorkTimeId(pathname: string): string | null {
	const match = /^\/hr\/employees\/([^/]+)\/work-time$/.exec(pathname)
	if (!match?.[1]) return null
	return decodeURIComponent(match[1])
}
