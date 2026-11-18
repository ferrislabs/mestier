import {
	Bug,
	Building2,
	FileText,
	LifeBuoy,
	ShieldCheck,
	ToggleRight,
	Workflow,
} from 'lucide-react'
import { AutomationSection } from '#/pages/settings/sections/automation-section'
import { DeveloperSection } from '#/pages/settings/sections/developer-section'
import { FeaturesSection } from '#/pages/settings/sections/features-section'
import { ModelsSection } from '#/pages/settings/sections/models-section'
import { OrganizationSection } from '#/pages/settings/sections/organization-section'
import { RolesSection } from '#/pages/settings/sections/roles-section'
import { SupportSection } from '#/pages/settings/sections/support-section'
import type { SettingsSection } from '#/pages/settings/types'

export const SETTINGS_SECTIONS: SettingsSection[] = [
	{
		id: 'organisation',
		label: 'Organisation',
		icon: Building2,
		Component: OrganizationSection,
	},
	{
		id: 'fonctionnalites',
		label: 'Fonctionnalités',
		icon: ToggleRight,
		Component: FeaturesSection,
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
		id: 'support',
		label: 'Support',
		icon: LifeBuoy,
		moduleId: 'support',
		Component: SupportSection,
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
