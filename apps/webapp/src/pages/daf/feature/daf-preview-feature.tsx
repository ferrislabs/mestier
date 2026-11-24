import { Link } from '@tanstack/react-router'
import { ArrowLeft } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	dafDossierPdfPath,
	dafPdfPath,
	downloadDafPdf,
	useDaf,
	useProjectDafs,
} from '#/hooks/use-daf'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { useDafPdfUrl } from '#/pages/daf/feature/use-daf-pdf-url'
import { dafNumberLabel } from '#/pages/daf/lib/daf-status'
import { DafPdfPreview } from '#/pages/daf/ui/daf-pdf-preview'

function usePdfDownload(path: string | null, filename: string) {
	const [isDownloading, setIsDownloading] = useState(false)
	const [error, setError] = useState<string | null>(null)
	const download = () => {
		if (!path) return
		setError(null)
		setIsDownloading(true)
		void downloadDafPdf(path, filename)
			.catch((failure) => setError(mutationErrorMessage(failure)))
			.finally(() => setIsDownloading(false))
	}
	return { isDownloading, error, download }
}

export function DafDossierPreviewFeature({ projectId }: { projectId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const listing = useProjectDafs(activeOrganization.id, projectId)
	const path = dafDossierPdfPath(activeOrganization.id, projectId)
	const pdf = useDafPdfUrl(path)
	const title = listing.data?.data.dossier.title ?? 'Dossier DAF'
	const download = usePdfDownload(path, `DAF-${title}.pdf`)

	return (
		<DafPdfPreview
			back={
				<Button variant="outline" size="sm" asChild>
					<Link
						to={buildOrgPath(activeOrganization.slug, '/projet/$projectId/daf')}
						params={{ projectId }}
					>
						<ArrowLeft aria-hidden /> Retour aux DAF
					</Link>
				</Button>
			}
			subtitle={`${title} · dossier complet`}
			pdfUrl={pdf.url}
			isLoading={pdf.isLoading}
			error={pdf.error ?? download.error}
			isDownloading={download.isDownloading}
			onDownload={download.download}
		/>
	)
}

export function DafSheetPreviewFeature({
	projectId,
	dafId,
}: {
	projectId: string
	dafId: string
}) {
	const { activeOrganization } = useActiveOrganization()
	const daf = useDaf(activeOrganization.id, dafId)
	const path = dafPdfPath(activeOrganization.id, dafId)
	const pdf = useDafPdfUrl(path)
	const loaded = daf.data?.data
	const label = loaded
		? `${dafNumberLabel(loaded.number)} – ${loaded.designation}`
		: 'DAF'
	const download = usePdfDownload(path, `${label}.pdf`)

	return (
		<DafPdfPreview
			back={
				<Button variant="outline" size="sm" asChild>
					<Link
						to={buildOrgPath(
							activeOrganization.slug,
							'/projet/$projectId/daf/$dafId',
						)}
						params={{ projectId, dafId }}
					>
						<ArrowLeft aria-hidden /> Retour à la DAF
					</Link>
				</Button>
			}
			subtitle={label}
			pdfUrl={pdf.url}
			isLoading={pdf.isLoading}
			error={pdf.error ?? download.error}
			isDownloading={download.isDownloading}
			onDownload={download.download}
		/>
	)
}
