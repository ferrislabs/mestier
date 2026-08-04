import { createFileRoute } from '@tanstack/react-router'
import { CrmConfigurationFeature } from '#/pages/crm-configuration/feature/crm-configuration-feature'

export const Route = createFileRoute('/_app/crm/configuration')({
	component: CrmConfigurationFeature,
})
