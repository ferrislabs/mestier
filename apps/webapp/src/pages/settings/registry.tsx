import { Building2, Workflow } from 'lucide-react'
import { AutomationSection } from '#/pages/settings/sections/automation-section'
import { OrganizationSection } from '#/pages/settings/sections/organization-section'
import type { SettingsSection } from '#/pages/settings/types'

export const SETTINGS_SECTIONS: SettingsSection[] = [
	{
		id: 'organisation',
		label: 'Organisation',
		icon: Building2,
		Component: OrganizationSection,
	},
	{
		id: 'automatisation',
		label: 'Automatisation',
		icon: Workflow,
		Component: AutomationSection,
	},
]
