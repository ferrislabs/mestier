import { createFileRoute } from '@tanstack/react-router'
import { DafEditorPageFeature } from '#/pages/daf/feature/daf-editor-page-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/projet/$projectId_/daf_/$dafId',
)({
	component: DafEditorPage,
})

function DafEditorPage() {
	const { projectId, dafId } = Route.useParams()
	return <DafEditorPageFeature projectId={projectId} dafId={dafId} />
}
