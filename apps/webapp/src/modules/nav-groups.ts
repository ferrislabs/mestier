import { GLOBAL_NAV_GROUPS } from '#/modules/registry'
import { resolveModule } from '#/modules/resolve-module'
import type { ModuleNavGroup } from '#/modules/types'

export function buildSidebarGroups(pathname: string): ModuleNavGroup[] {
	return [...resolveModule(pathname).nav, ...GLOBAL_NAV_GROUPS]
}
