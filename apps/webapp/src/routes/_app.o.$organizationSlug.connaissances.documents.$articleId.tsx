import { createFileRoute, lazyRouteComponent } from '@tanstack/react-router'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/connaissances/documents/$articleId',
)({
	component: lazyRouteComponent(
		() => import('#/pages/knowledge-documents/feature/document-page-route'),
		'DocumentRoutePage',
	),
})
