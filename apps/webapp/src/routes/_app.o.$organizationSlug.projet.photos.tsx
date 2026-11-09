import { createFileRoute } from '@tanstack/react-router'
import { PhotoLibraryFeature } from '#/pages/photo-library/feature/photo-library-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/projet/photos')(
	{
		component: PhotoLibraryFeature,
	},
)
