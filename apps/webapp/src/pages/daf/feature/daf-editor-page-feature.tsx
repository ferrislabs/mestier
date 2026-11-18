import { Link } from '@tanstack/react-router'
import { AlertCircle, ArrowLeft, Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useDaf, useProjectDafs } from '#/hooks/use-daf'
import { useProject } from '#/hooks/use-projects'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { DafEditorFeature } from '#/pages/daf/feature/daf-editor-feature'

export function DafEditorPageFeature({
	projectId,
	dafId,
}: {
	projectId: string
	dafId: string
}) {
	const { activeOrganization } = useActiveOrganization()
	const project = useProject(activeOrganization.id, projectId)
	const daf = useDaf(activeOrganization.id, dafId)
	const listing = useProjectDafs(activeOrganization.id, projectId)
	const dossier = listing.data?.data.dossier

	if (daf.isLoading || listing.isLoading) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement de la DAF…
				</SectionCard>
			</PageShell>
		)
	}

	if (!daf.data?.data || !dossier) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
					<AlertCircle className="size-6 text-destructive" />
					<div>
						<p className="font-semibold">Impossible de charger la DAF</p>
						<p className="mt-1 text-sm text-muted-foreground">
							{mutationErrorMessage(daf.error ?? listing.error) ??
								'DAF introuvable.'}
						</p>
					</div>
					<Button asChild variant="outline">
						<Link
							to={buildOrgPath(
								activeOrganization.slug,
								'/projet/$projectId/daf',
							)}
							params={{ projectId }}
						>
							<ArrowLeft />
							Retour aux DAF
						</Link>
					</Button>
				</SectionCard>
			</PageShell>
		)
	}

	return (
		<DafEditorFeature
			key={daf.data.data.id}
			daf={daf.data.data}
			dossier={dossier}
			organizationId={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			projectId={projectId}
			projectName={project.data?.data.name ?? 'Projet'}
		/>
	)
}
