import { createFileRoute } from '@tanstack/react-router'
import { HomeFeature } from '#/pages/home/feature/home-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/')({
	component: HomeFeature,
})
