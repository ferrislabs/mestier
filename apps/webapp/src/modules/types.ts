import type { LucideIcon } from 'lucide-react'
import type { PermissionName } from '#/hooks/use-permissions'

export type ModuleId =
	| 'home'
	| 'crm'
	| 'purchase'
	| 'hr'
	| 'planning'
	| 'chat'
	| 'settings'

/**
 * `coming-soon`: module announced but not navigable — it stays visible, greyed
 * out, in the module launcher.
 * `hidden`: never rendered. Reserved for future filtering by rights, out of scope.
 */
export type ModuleStatus = 'available' | 'coming-soon' | 'hidden'

export type NavStatus = Exclude<ModuleStatus, 'hidden'>

/** Navigation target, shared by both levels. */
export interface NavTarget {
	id: string
	label: string
	to: string
	icon?: LucideIcon
	/** Active on an exact path match only. */
	exact?: boolean
	status?: NavStatus
	badge?: string | number
	/**
	 * #307: hidden entirely, not greyed out, for a caller who does not hold
	 * this bit — unlike `status: 'coming-soon'`, which is a placeholder
	 * everyone sees. Presentation only: the API refuses regardless of
	 * whether this hides the link, see `hooks/use-permissions.ts`'s own
	 * doc comment.
	 */
	requiredPermission?: PermissionName
}

/** Second level: horizontal tabs, under the breadcrumb. */
export type ModuleTab = NavTarget

/** First level: entries of the module's side nav. */
export interface ModuleSection extends NavTarget {
	tabs?: ModuleTab[]
}

export interface AppModule {
	id: ModuleId
	label: string
	icon: LucideIcon
	basePath: string
	status: ModuleStatus
	/** Intra-module navigation, rendered in the left column. */
	sections: ModuleSection[]
	/**
	 * `basePath` renders an overview instead of redirecting to the first
	 * navigable section.
	 */
	hasOverview: boolean
	/**
	 * `utility` puts the module at the foot of the nav, apart from the business
	 * modules. Defaults to `primary`.
	 */
	railPlacement?: 'primary' | 'utility'
}
