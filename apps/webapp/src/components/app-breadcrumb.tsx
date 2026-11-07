import { Link, useLocation } from '@tanstack/react-router'
import { Fragment, type ReactNode } from 'react'
import { OrgSwitcher } from '#/components/org-switcher'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useCustomer } from '#/hooks/use-customers'
import { useMember } from '#/hooks/use-reference-catalog'
import { buildBreadcrumbItems } from '#/modules/breadcrumb'
import { BreadcrumbSeparator } from '#/modules/breadcrumb-parts'
import { splitOrgPath } from '#/modules/org-path'
import { customerDisplayName } from '#/pages/customers/types'
import { DoeModelCrumbFeature } from '#/pages/projet/feature/doe-model-crumb-feature'
import { ProjetCrumbsFeature } from '#/pages/projet/feature/projet-crumbs-feature'
import { doeModelIdOf, projectIdOf } from '#/pages/projet/lib/project-path'

interface AppBreadcrumbProps {
	/** Brand and launcher on a screen without side nav, ahead of the organization. */
	leading?: ReactNode
}

export function AppBreadcrumb({ leading }: AppBreadcrumbProps) {
	const location = useLocation()
	const { activeOrganization } = useActiveOrganization()
	// The paths matched below are relative to the organization: the tenant is
	// stripped before any matching.
	const modulePath = splitOrgPath(location.pathname).path
	const customerId = getCustomerId(modulePath)
	const customer = useCustomer(customerId ?? '', Boolean(customerId))
	const customerLabel = customer.data?.data
		? customerDisplayName(customer.data.data)
		: 'Fiche client'

	const memberId = getMemberWorkTimeId(modulePath)
	const member = useMember(memberId ?? '', Boolean(memberId))
	const memberLabel = member.data?.data
		? member.data.data.display_name
		: 'Temps de travail'

	const projectId = projectIdOf(modulePath)
	const modelId = doeModelIdOf(modulePath)
	const hasTail = projectId !== null || modelId !== null

	const items = buildBreadcrumbItems({
		pathname: location.pathname,
		organizationName: activeOrganization.name,
		organizationSlug: activeOrganization.slug,
		detailLabel: getDetailLabel(modulePath, customerLabel, memberLabel),
	})

	return (
		<nav
			aria-label="Fil d’Ariane"
			className="flex min-w-0 items-center gap-1.5 text-[13px]"
		>
			{leading ? (
				<>
					{leading}
					<BreadcrumbSeparator />
				</>
			) : null}
			{items.map((item, index) => {
				const isEnd = index === items.length - 1
				const isLast = isEnd && !hasTail

				return (
					<Fragment key={item.id}>
						{item.id === 'organization' ? (
							<OrgSwitcher />
						) : isLast || !item.to ? (
							<span aria-current="page" className="truncate font-medium">
								{item.label}
							</span>
						) : (
							<Link
								to={item.to}
								className="truncate rounded-sm text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
							>
								{item.label}
							</Link>
						)}
						{isEnd ? null : <BreadcrumbSeparator />}
					</Fragment>
				)
			})}
			{projectId ? <ProjetCrumbsFeature /> : null}
			{modelId ? <DoeModelCrumbFeature modelId={modelId} /> : null}
		</nav>
	)
}

function getDetailLabel(
	pathname: string,
	customerLabel: string,
	memberLabel: string,
): string | undefined {
	if (getCustomerId(pathname)) return customerLabel
	if (/^\/crm\/quotes\/[^/]+$/.test(pathname)) return 'Fiche devis'
	if (pathname === '/projet/new') return 'Nouveau projet'
	if (getMemberWorkTimeId(pathname)) return memberLabel
	return undefined
}

function getCustomerId(pathname: string): string | null {
	if (pathname === '/crm/pipeline') return null
	const match = /^\/crm\/customers\/([^/]+)$/.exec(pathname)
	if (!match?.[1]) return null
	return decodeURIComponent(match[1])
}

function getMemberWorkTimeId(pathname: string): string | null {
	const match = /^\/hr\/team\/([^/]+)\/work-time$/.exec(pathname)
	if (!match?.[1]) return null
	return decodeURIComponent(match[1])
}
