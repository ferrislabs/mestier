import { useLocation } from '@tanstack/react-router'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { usePermissions } from '#/hooks/use-permissions'
import { splitOrgPath } from '#/modules/org-path'
import {
	useEstablishments,
	useTrainings,
} from '#/pages/cursus/catalog/feature/use-cursus-catalog'
import { useHostCompanies } from '#/pages/cursus/host-companies/feature/use-host-companies'
import { useIsCursusParticipant } from '#/pages/cursus/me/feature/use-cursus-participation'
import { countOf } from '#/pages/cursus/nav/lib/count-of'
import { activeCursusEntry } from '#/pages/cursus/nav/lib/cursus-path'
import { CursusSidebar } from '#/pages/cursus/nav/ui/cursus-sidebar'
import { useParticipants } from '#/pages/cursus/participants/feature/use-participants'
import { usePromotions } from '#/pages/cursus/promotions/feature/use-promotions'
import { useTriads } from '#/pages/cursus/triads/feature/use-triads'
import { useSidebarGroups } from '#/pages/projet/feature/use-sidebar-groups'

const COLLAPSED_KEY = 'mestier:cursus:sidebar-collapsed'

export function CursusSidebarFeature() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const [isCollapsed, toggleGroup] = useSidebarGroups(COLLAPSED_KEY)
	const pathname = useLocation({ select: (current) => current.pathname })

	const permissions = usePermissions()
	const canView =
		permissions.data?.data.permissions.includes('VIEW_CURSUS') ?? false
	const isParticipant = useIsCursusParticipant()

	const promotions = usePromotions(
		organizationId,
		{ archived: 'active', trainingId: undefined },
		canView,
	)
	const triads = useTriads(
		organizationId,
		{ promotionId: undefined, closed: false },
		canView,
	)
	const participants = useParticipants(organizationId, { per_page: 1 }, canView)
	const companies = useHostCompanies(organizationId, { per_page: 1 }, canView)
	const trainings = useTrainings(organizationId, {}, canView)
	const establishments = useEstablishments(organizationId, {}, canView)

	return (
		<CursusSidebar
			organizationSlug={activeOrganization.slug}
			active={activeCursusEntry(splitOrgPath(pathname).path)}
			counts={{
				promotions: countOf(promotions.data),
				triads: countOf(triads.data),
				participants: countOf(participants.data),
				companies: countOf(companies.data),
				trainings: countOf(trainings.data),
				establishments: countOf(establishments.data),
			}}
			canView={canView}
			isParticipant={isParticipant}
			isCollapsed={isCollapsed}
			onToggleGroup={toggleGroup}
		/>
	)
}
