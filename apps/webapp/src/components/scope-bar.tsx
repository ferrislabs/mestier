import type * as React from 'react'

import { NavTabs } from '#/components/nav-tabs'
import type { ModuleTab } from '#/modules/types'

interface ScopeBarProps {
	label: string
	tabs: ModuleTab[]
	organizationSlug: string
	/**
	 * Actions of the current scope — saving a record, creating an entity. They
	 * live here rather than in a bar floating above the content.
	 */
	actions?: React.ReactNode
}

/**
 * Navigation bar for the active scope, sitting under the header.
 *
 * A scope exposing a single screen shows no bar: the rail and the breadcrumb
 * are enough to tell the user where they are.
 */
export function ScopeBar({
	label,
	tabs,
	organizationSlug,
	actions,
}: ScopeBarProps) {
	if (tabs.length <= 1 && !actions) return null

	return (
		<div className="sticky top-(--app-header-height) z-10 flex items-center gap-4 border-b bg-card px-3 md:px-6">
			<NavTabs label={label} tabs={tabs} organizationSlug={organizationSlug} />
			{actions ? (
				<div className="flex shrink-0 items-center gap-2 py-2">{actions}</div>
			) : null}
		</div>
	)
}
