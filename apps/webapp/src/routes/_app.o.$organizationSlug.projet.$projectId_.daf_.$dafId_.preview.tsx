import { createFileRoute } from '@tanstack/react-router'
import { DafSheetPreviewFeature } from '#/pages/daf/feature/daf-preview-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/projet/$projectId_/daf_/$dafId_/preview',
)({
	component: DafSheetPreviewPage,
})

function DafSheetPreviewPage() {
	const { projectId, dafId } = Route.useParams()
	return <DafSheetPreviewFeature projectId={projectId} dafId={dafId} />
}
