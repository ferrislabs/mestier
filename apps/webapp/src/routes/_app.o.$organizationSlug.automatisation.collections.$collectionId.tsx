import { createFileRoute } from '@tanstack/react-router'
import { CollectionDetailFeature } from '#/pages/automation/feature/collection-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/automatisation/collections/$collectionId',
)({
	component: CollectionPage,
})

function CollectionPage() {
	const { collectionId } = Route.useParams()

	return <CollectionDetailFeature collectionId={collectionId} />
}
