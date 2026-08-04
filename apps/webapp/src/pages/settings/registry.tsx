import { Building2, Package } from 'lucide-react'
import { EquipmentSection } from '#/pages/settings/sections/equipment-section'
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
		id: 'equipement',
		label: 'Matériel',
		icon: Package,
		Component: EquipmentSection,
	},
]
