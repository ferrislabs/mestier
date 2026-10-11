import { Link } from '@tanstack/react-router'
import type { ColumnDef } from '@tanstack/react-table'
import { FilePlus2, FileText, Loader2, Plus, Trash2 } from 'lucide-react'
import { useMemo } from 'react'
import { ReferenceTable } from '#/components/reference-table'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
} from '#/components/ui/surface'
import type { DoeSummary } from '#/hooks/use-doe'
import { buildOrgPath } from '#/modules/org-path'
import { formatIssuedOn, formatUpdatedAt } from '#/pages/does/lib/format'

interface DoesListProps {
	organizationName: string
	organizationSlug: string
	does: DoeSummary[]
	isLoading: boolean
	error: string | null
	onNewDoe: () => void
	onDelete: (doe: DoeSummary) => void
}

export function DoesList({
	organizationName,
	organizationSlug,
	does,
	isLoading,
	error,
	onNewDoe,
	onDelete,
}: DoesListProps) {
	const columns = useMemo<ColumnDef<DoeSummary>[]>(() => {
		const open = (doe: DoeSummary, label: string, className?: string) => (
			<Link
				to={buildOrgPath(
					organizationSlug,
					'/planification/projects/$projectId/doe',
				)}
				params={{ projectId: doe.project_id }}
				className={className ?? 'block hover:underline'}
			>
				{label}
			</Link>
		)
		return [
			{
				id: 'project',
				header: 'Projet',
				cell: ({ row }) =>
					open(row.original, row.original.project_name, 'block font-medium'),
			},
			{
				id: 'title',
				header: 'Titre',
				cell: ({ row }) => open(row.original, row.original.title),
			},
			{
				id: 'revision',
				header: 'Indice',
				cell: ({ row }) => <span>{row.original.revision}</span>,
			},
			{
				id: 'issued_on',
				header: 'Date',
				cell: ({ row }) => (
					<span>{formatIssuedOn(row.original.issued_on)}</span>
				),
			},
			{
				id: 'updated_at',
				header: 'Modifié',
				cell: ({ row }) => (
					<span>{formatUpdatedAt(row.original.updated_at)}</span>
				),
			},
			{
				id: 'actions',
				header: () => <span className="sr-only">Actions</span>,
				cell: ({ row }) => (
					<div className="flex justify-end">
						<RequirePermission permission="MANAGE_PLANNING">
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								onClick={() => onDelete(row.original)}
							>
								<Trash2 />
								<span className="sr-only">
									Supprimer le DOE {row.original.title}
								</span>
							</Button>
						</RequirePermission>
					</div>
				),
			},
		]
	}, [organizationSlug, onDelete])

	const newButton = (
		<RequirePermission permission="MANAGE_PLANNING">
			<Button onClick={onNewDoe}>
				<Plus />
				Nouveau DOE
			</Button>
		</RequirePermission>
	)

	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="DOE"
				description="Le dossier des ouvrages exécutés d'un projet : fiches techniques, photos et plans remis au client."
				actions={!isLoading && does.length === 0 ? undefined : newButton}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des DOE…
				</SectionCard>
			) : does.length === 0 ? (
				<SectionCard className="flex flex-col items-center gap-4 p-10 text-center">
					<FilePlus2 className="size-8 text-muted-foreground" />
					<div>
						<p className="font-semibold">Aucun DOE pour le moment</p>
						<p className="mt-1 text-sm text-muted-foreground">
							Un DOE est le dossier des ouvrages exécutés que vous remettez au
							client en fin de chantier, construit à partir d'un modèle.
						</p>
					</div>
					{newButton}
				</SectionCard>
			) : (
				<>
					<section className="grid grid-cols-2 gap-4 lg:grid-cols-3">
						<MetricCard
							label="DOE"
							value={does.length}
							icon={<FileText className="size-4" />}
						/>
					</section>
					<ReferenceTable
						title="DOE"
						description="Les plus récemment modifiés en premier. Ouvrez un DOE pour l'éditer."
						emptyTitle="Aucun DOE"
						emptyDescription="Aucun DOE à afficher."
						data={does}
						columns={columns}
					/>
				</>
			)}
		</PageShell>
	)
}
