import { Link } from '@tanstack/react-router'
import { AlertCircle, ArrowLeft, Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageShell, SectionCard } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { isNotFoundError, useProjectDoe } from '#/hooks/use-doe'
import { useProject } from '#/hooks/use-projects'
import { buildOrgPath } from '#/modules/org-path'
import { DoeCreateFeature } from '#/pages/doe/feature/doe-create-feature'
import { DoeEditorFeature } from '#/pages/doe/feature/doe-editor-feature'

interface DoeFeatureProps {
	projectId: string
}

export function DoeFeature({ projectId }: DoeFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const project = useProject(activeOrganization.id, projectId)
	const doe = useProjectDoe(activeOrganization.id, projectId)
	const projectName = project.data?.data.name ?? 'Projet'

	if (doe.isLoading) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement du DOE…
				</SectionCard>
			</PageShell>
		)
	}

	if (doe.isError && !isNotFoundError(doe.error)) {
		return (
			<PageShell>
				<SectionCard className="flex min-h-72 flex-col items-center justify-center gap-3 p-8 text-center">
					<AlertCircle className="size-6 text-destructive" />
					<div>
						<p className="font-semibold">Impossible de charger le DOE</p>
						<p className="mt-1 text-sm text-muted-foreground">
							{doe.error.message}
						</p>
					</div>
					<div className="flex gap-2">
						<Button asChild variant="outline">
							<Link
								to={buildOrgPath(
									activeOrganization.slug,
									'/planification/projects',
								)}
							>
								<ArrowLeft />
								Retour aux projets
							</Link>
						</Button>
						<Button type="button" onClick={() => void doe.refetch()}>
							Réessayer
						</Button>
					</div>
				</SectionCard>
			</PageShell>
		)
	}

	if (doe.data?.data) {
		return (
			<DoeEditorFeature
				key={doe.data.data.id}
				doe={doe.data.data}
				organizationId={activeOrganization.id}
				organizationSlug={activeOrganization.slug}
				projectId={projectId}
				projectName={projectName}
			/>
		)
	}

	return (
		<PageShell>
			<DoeCreateFeature
				organizationId={activeOrganization.id}
				projectId={projectId}
				projectName={projectName}
			/>
		</PageShell>
	)
}
