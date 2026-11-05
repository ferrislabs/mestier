import { Link } from '@tanstack/react-router'
import { ArrowLeft, Download, Loader2 } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import { Page, PageHeader } from '#/pages/projects/ui/page'

export interface DoePreviewProps {
	organizationSlug: string
	projectId: string
	projectName: string | null
	doe: { title: string; revision: string } | null
	pdfUrl: string | null
	isLoading: boolean
	error: string | null
	isDownloading: boolean
	onDownload: () => void
}

export function DoePreview({
	organizationSlug,
	projectId,
	projectName,
	doe,
	pdfUrl,
	isLoading,
	error,
	isDownloading,
	onDownload,
}: DoePreviewProps) {
	const projectsPath = buildOrgPath(organizationSlug, '/projet')
	const editorPath = buildOrgPath(organizationSlug, '/projet/$projectId/doe')

	if (projectName === null) {
		return (
			<Page>
				<PageHeader title="Aperçu du PDF" />
				<EmptyState
					title="Projet introuvable"
					hint="Ce projet n’existe plus ou le lien n’est pas correct."
				>
					<Button variant="outline" size="sm" asChild>
						<Link to={projectsPath}>
							<ArrowLeft aria-hidden /> Voir les projets
						</Link>
					</Button>
				</EmptyState>
			</Page>
		)
	}

	if (!doe) {
		return (
			<Page>
				<PageHeader title="Aperçu du PDF" description={projectName} />
				<EmptyState
					title="Ce projet n’a pas encore de DOE"
					hint="Crée le DOE du projet pour voir l’aperçu de son PDF."
				>
					<Button variant="outline" size="sm" asChild>
						<Link to={editorPath} params={{ projectId }}>
							<ArrowLeft aria-hidden /> Aller au DOE
						</Link>
					</Button>
				</EmptyState>
			</Page>
		)
	}

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<div className="sticky top-0 z-10 flex flex-wrap items-center gap-x-4 gap-y-2 border-b bg-background/95 px-6 py-3 backdrop-blur">
				<Button variant="outline" size="sm" asChild>
					<Link to={editorPath} params={{ projectId }}>
						<ArrowLeft aria-hidden /> Retour à l’éditeur
					</Link>
				</Button>
				<div className="min-w-0 flex-1">
					<h1 className="truncate font-semibold text-lg leading-tight">
						Aperçu du PDF
					</h1>
					<p className="truncate text-[13px] text-muted-foreground">
						{doe.title} · indice {doe.revision}
					</p>
				</div>
				<Button type="button" disabled={isDownloading} onClick={onDownload}>
					{isDownloading ? (
						<Loader2 className="animate-spin" aria-hidden />
					) : (
						<Download aria-hidden />
					)}
					Télécharger le PDF
				</Button>
			</div>
			<div className="flex min-h-0 flex-1 flex-col bg-muted/60">
				{isLoading ? (
					<div className="flex min-h-48 flex-1 items-center justify-center gap-3 text-muted-foreground text-sm">
						<Loader2 className="size-5 animate-spin" aria-hidden />
						Génération du PDF…
					</div>
				) : error || !pdfUrl ? (
					<p
						role="alert"
						className="m-6 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
					>
						{error ?? 'Le PDF est indisponible.'}
					</p>
				) : (
					<iframe
						title={`Aperçu du PDF du DOE ${doe.title}`}
						src={pdfUrl}
						className="min-h-[70vh] w-full flex-1 border-0"
					/>
				)}
			</div>
		</div>
	)
}
