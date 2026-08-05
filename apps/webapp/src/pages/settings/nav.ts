import { MODULES } from '#/modules/registry'
import type { SettingsSection } from '#/pages/settings/types'

export interface SettingsNavGroup {
	label: string
	sections: SettingsSection[]
}

export function buildSettingsNavGroups(
	sections: SettingsSection[],
): SettingsNavGroup[] {
	const groups: SettingsNavGroup[] = []

	const general = sections.filter((section) => !section.moduleId)
	if (general.length > 0) groups.push({ label: 'Général', sections: general })

	for (const module of MODULES) {
		const owned = sections.filter((section) => section.moduleId === module.id)
		if (owned.length > 0) groups.push({ label: module.label, sections: owned })
	}

	return groups
}
