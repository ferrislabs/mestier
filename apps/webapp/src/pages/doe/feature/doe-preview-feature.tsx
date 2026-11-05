import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { downloadDoePdf, isNotFoundError, useProjectDoe } from '#/hooks/use-doe'
import { useProject } from '#/hooks/use-projects'
import { mutationErrorMessage } from '#/lib/api-error'
import { useDoePdfUrl } from '#/pages/doe/feature/use-doe-pdf-url'
import { DoePreview } from '#/pages/doe/ui/doe-preview'
import { PageLoading } from '#/pages/projects/ui/page-loading'

export function DoePreviewFeature({ projectId }: { projectId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const project = useProject(activeOrganization.id, projectId)
	const doe = useProjectDoe(activeOrganization.id, projectId)
	const pdf = useDoePdfUrl(activeOrganization.id, doe.data?.data.id ?? '')
	const [isDownloading, setIsDownloading] = useState(false)
	const [downloadError, setDownloadError] = useState<string | null>(null)

	if (project.isLoading || doe.isLoading)
		return <PageLoading label="Chargement de l’aperçu…" />

	const missing = doe.isError && isNotFoundError(doe.error)
	const loaded = doe.data?.data

	return (
		<DoePreview
			organizationSlug={activeOrganization.slug}
			projectId={projectId}
			projectName={project.data?.data.name ?? null}
			doe={
				!missing && loaded
					? { title: loaded.title, revision: loaded.revision }
					: null
			}
			pdfUrl={pdf.url}
			isLoading={pdf.isLoading}
			error={pdf.error ?? downloadError}
			isDownloading={isDownloading}
			onDownload={() => {
				if (!loaded) return
				setDownloadError(null)
				setIsDownloading(true)
				void downloadDoePdf(
					activeOrganization.id,
					loaded.id,
					`DOE-${loaded.title}.pdf`,
				)
					.catch((failure) => setDownloadError(mutationErrorMessage(failure)))
					.finally(() => setIsDownloading(false))
			}}
		/>
	)
}
