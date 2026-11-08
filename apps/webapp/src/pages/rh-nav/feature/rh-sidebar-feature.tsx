import { useLocation } from '@tanstack/react-router'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { type PermissionName, usePermissions } from '#/hooks/use-permissions'
import { splitOrgPath } from '#/modules/org-path'
import { useRhModel } from '#/pages/hr/feature/use-rh-model'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import { useSidebarGroups } from '#/pages/projet/feature/use-sidebar-groups'
import { activeSidebarEntries } from '#/pages/rh-nav/lib/sidebar-active'
import { sidebarCounts } from '#/pages/rh-nav/lib/sidebar-counts'
import { sidebarDeadlines } from '#/pages/rh-nav/lib/sidebar-deadlines'
import { RhSidebar } from '#/pages/rh-nav/ui/rh-sidebar'

const COLLAPSED_KEY = 'mestier:rh:sidebar-collapsed'
const DEADLINE_LIMIT = 5

export function RhSidebarFeature() {
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
	const model = useRhModel(activeOrganization.id, activeOrganization.slug)

	const granted: readonly string[] = permissions.data?.data.permissions ?? []
	const visible = (permission: PermissionName | null) =>
		permission === null || granted.includes(permission)

	const ready = !model.isLoading && model.error === null
	const accounts = ready
		? model.state.accounts.filter((account) => !account.archived)
		: []
	const currentMemberId = model.context.actorId

	return (
		<RhSidebar
			organizationSlug={activeOrganization.slug}
			active={activeSidebarEntries(
				modulePath,
				location.search,
				currentMemberId,
				accounts[0]?.id,
			)}
			counts={ready ? sidebarCounts(model.state, currentMemberId) : null}
			currentMemberId={currentMemberId}
			accounts={accounts}
			deadlines={
				ready ? sidebarDeadlines(model.state, todayIso(), DEADLINE_LIMIT) : []
			}
			visible={visible}
			isCollapsed={isCollapsed}
			onToggleGroup={toggleGroup}
		/>
	)
}
