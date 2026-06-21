import { createFileRoute } from '@tanstack/react-router'
import { CatalogFeature } from '#/pages/catalog/feature/catalog-feature'

export const Route = createFileRoute('/_app/catalog/')({
	component: CatalogFeature,
})
