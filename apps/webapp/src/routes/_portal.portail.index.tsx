import { createFileRoute } from '@tanstack/react-router'
import { PortalSpacesFeature } from '#/pages/cursus/portal/feature/portal-spaces-feature'

export const Route = createFileRoute('/_portal/portail/')({
	component: PortalSpacesFeature,
})
