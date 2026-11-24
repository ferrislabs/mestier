import { Link } from '@tanstack/react-router'
import { FileText, Loader2, Plus } from 'lucide-react'
import { useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { useProjectDafs } from '#/hooks/use-daf'
import { mutationErrorMessage } from '#/lib/api-error'
import { buildOrgPath } from '#/modules/org-path'
import { DafCreateFeature } from '#/pages/daf/feature/daf-create-feature'
import { DafDossierFeature } from '#/pages/daf/feature/daf-dossier-feature'
import { DafEmptyState } from '#/pages/daf/ui/daf-empty-state'
import { DafSummaryTable } from '#/pages/daf/ui/daf-summary-table'
import { EditorCard } from '#/pages/doe/ui/editor-card'

export function DafListFeature({
	organizationId,
	organizationSlug,
	projectId,
}: {
	organizationId: string
	organizationSlug: string
	projectId: string
}) {
	const listing = useProjectDafs(organizationId, projectId)
	const [createOpen, setCreateOpen] = useState(false)

	if (listing.isLoading) {
		return (
			<div className="flex items-center gap-2 py-10 text-muted-foreground text-sm">
				<Loader2 className="size-4 animate-spin" aria-hidden />
				Chargement des DAF…
			</div>
		)
	}

	if (listing.isError || !listing.data) {
		return (
			<div role="alert" className="grid justify-items-start gap-3 py-6">
				<p className="text-destructive text-sm">
					{mutationErrorMessage(listing.error) ??
						'Impossible de charger les DAF.'}
				</p>
				<Button type="button" onClick={() => void listing.refetch()}>
					Réessayer
				</Button>
			</div>
		)
	}

	const { dossier, dafs } = listing.data.data

	return (
		<div className="grid gap-6">
			<DafDossierFeature
				key={dossier.updated_at ?? 'default'}
				organizationId={organizationId}
				projectId={projectId}
				dossier={dossier}
			/>
			<EditorCard
				title="Sommaire"
				description="Généré à partir des DAF du projet."
			>
				<div className="flex flex-wrap items-center gap-2 border-b px-5 py-3">
					<RequirePermission permission="MANAGE_PLANNING">
						<Button type="button" size="sm" onClick={() => setCreateOpen(true)}>
							<Plus aria-hidden />
							Nouvelle DAF
						</Button>
					</RequirePermission>
					{dafs.length === 0 ? (
						<Button type="button" variant="outline" size="sm" disabled>
							<FileText aria-hidden />
							PDF du dossier
						</Button>
					) : (
						<Button variant="outline" size="sm" asChild>
							<Link
								to={buildOrgPath(
									organizationSlug,
									'/projet/$projectId/daf/preview',
								)}
								params={{ projectId }}
							>
								<FileText aria-hidden />
								PDF du dossier
							</Link>
						</Button>
					)}
				</div>
				{dafs.length === 0 ? (
					<DafEmptyState />
				) : (
					<DafSummaryTable
						organizationSlug={organizationSlug}
						projectId={projectId}
						rows={dafs}
					/>
				)}
			</EditorCard>
			<DafCreateFeature
				open={createOpen}
				organizationId={organizationId}
				organizationSlug={organizationSlug}
				projectId={projectId}
				onClose={() => setCreateOpen(false)}
			/>
		</div>
	)
}
