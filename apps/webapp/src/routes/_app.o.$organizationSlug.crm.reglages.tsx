import { createFileRoute } from '@tanstack/react-router'
import { SettingsFeature } from '#/pages/crm-settings/feature/settings-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/reglages')({
	component: SettingsFeature,
})
