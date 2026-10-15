import { Bug, Building2, FileText, ShieldCheck, Workflow } from 'lucide-react'
import { AutomationSection } from '#/pages/settings/sections/automation-section'
import { DeveloperSection } from '#/pages/settings/sections/developer-section'
import { ModelsSection } from '#/pages/settings/sections/models-section'
import { OrganizationSection } from '#/pages/settings/sections/organization-section'
import { RolesSection } from '#/pages/settings/sections/roles-section'
import type { SettingsSection } from '#/pages/settings/types'

export const SETTINGS_SECTIONS: SettingsSection[] = [
	{
		id: 'organisation',
		label: 'Organisation',
		icon: Building2,
		Component: OrganizationSection,
	},
	{
		id: 'roles',
		label: 'Rôles',
		icon: ShieldCheck,
		Component: RolesSection,
	},
	{
		id: 'automatisation',
		label: 'Automatisation',
		icon: Workflow,
		Component: AutomationSection,
	},
	{
		id: 'modeles',
		label: 'Modèles',
		icon: FileText,
		Component: ModelsSection,
	},
	{
		id: 'developpeur',
		label: 'Développeur',
		icon: Bug,
		Component: DeveloperSection,
	},
]
