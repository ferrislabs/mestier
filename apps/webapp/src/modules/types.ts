import type { LucideIcon } from 'lucide-react'
import type { ComponentType } from 'react'
import type { PermissionName } from '#/hooks/use-permissions'

export type ModuleId =
	| 'home'
	| 'crm'
	| 'projet'
	| 'purchase'
	| 'hr'
	| 'planning'
	| 'planification'
	| 'chat'
	| 'automatisation'
	| 'support'
	| 'cursus'
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
	/**
	 * Visible only to a person holding a participant seat in the active
	 * organization, whatever their permission bits (ADR 0015). Takes over from
	 * `requiredPermission` when both are set.
	 */
	visibleWhen?: 'cursus-participant'
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
	/**
	 * Hue (0-360) of the module's mark in the launcher and the side nav. Colour
	 * identity only: the lightness and chroma come from the active theme.
	 */
	hue?: number
	basePath: string
	status: ModuleStatus
	/** Intra-module navigation, rendered in the left column. */
	sections: ModuleSection[]
	/**
	 * `basePath` renders an overview instead of redirecting to the first
	 * navigable section.
	 */
	hasOverview: boolean
	/** One line shown next to the module in the search palette. */
	description?: string
	/**
	 * `false` for a place rather than a module to switch to (the organization
	 * home, the settings): it gets no tile in the launcher. Defaults to `true`.
	 */
	inLauncher?: boolean
	/**
	 * `false` for a module that lives without a side nav: the header then
	 * carries the brand and the launcher. Defaults to `true`.
	 */
	sideNav?: boolean
	/**
	 * Replaces the sections list in the side nav while it is expanded (e.g. a
	 * project switcher). The sections stay the source of truth for the
	 * collapsed rail, the breadcrumb and the landing path.
	 */
	sidebar?: ComponentType
	/** Extra block under the module's sections in the side nav (e.g. a folder tree). */
	sidebarExtra?: ComponentType
	/** Call-to-action pinned under the module's mark in the side nav. */
	primaryAction?: {
		label: string
		to: string
		search?: Readonly<Record<string, string>>
		/** Replaces the "+" that precedes the label. */
		icon?: LucideIcon
		/** Drawn after the label, e.g. a count of what waits. */
		badge?: ComponentType
	}
	requiredPermission?: PermissionName
}
