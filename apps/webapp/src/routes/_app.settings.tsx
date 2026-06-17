import { createFileRoute } from '@tanstack/react-router'
import { SettingsFeature } from '#/pages/settings/feature/settings-feature'

export const Route = createFileRoute('/_app/settings')({
	component: SettingsFeature,
})
