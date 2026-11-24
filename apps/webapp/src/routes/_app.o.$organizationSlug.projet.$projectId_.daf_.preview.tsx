import { createFileRoute } from '@tanstack/react-router'
import { DafDossierPreviewFeature } from '#/pages/daf/feature/daf-preview-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/projet/$projectId_/daf_/preview',
)({
	component: DafDossierPreviewPage,
})

function DafDossierPreviewPage() {
	const { projectId } = Route.useParams()
	return <DafDossierPreviewFeature projectId={projectId} />
}
