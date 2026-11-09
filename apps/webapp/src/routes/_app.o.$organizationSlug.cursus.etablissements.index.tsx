import { createFileRoute } from '@tanstack/react-router'
import { EstablishmentsPageFeature } from '#/pages/cursus/catalog/feature/establishments-page-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/etablissements/',
)({
	component: EstablishmentsPageFeature,
})
