import { createFileRoute } from '@tanstack/react-router'
import { useCallback } from 'react'
import { KnowledgeFeature } from '#/pages/knowledge/feature/knowledge-feature'
import { driveSearchSchema } from '#/pages/knowledge/lib/folders'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/connaissances/',
)({
	validateSearch: (search) => driveSearchSchema.parse(search),
	component: DrivePage,
})

function DrivePage() {
	const { dossier } = Route.useSearch()
	const navigate = Route.useNavigate()
	const onFolderChange = useCallback(
		(folderId: string | null) =>
			navigate({ search: folderId ? { dossier: folderId } : {} }),
		[navigate],
	)
	return (
		<KnowledgeFeature
			folderId={dossier ?? null}
			onFolderChange={onFolderChange}
		/>
	)
}
