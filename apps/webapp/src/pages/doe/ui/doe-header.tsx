import { Link } from '@tanstack/react-router'
import { Download, Eye, Loader2, Save } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'

interface DoeHeaderProps {
	organizationSlug: string
	projectId: string
	projectName: string
	title: string
	revision: string
	lot: string
	isDirty: boolean
	isSaving: boolean
	isDownloading: boolean
	canSave: boolean
	notice: string | null
	onSave: () => void
	onPreview: () => void
	onDownload: () => void
}

export function DoeHeader({
	organizationSlug,
	projectId,
	projectName,
	title,
	revision,
	lot,
	isDirty,
	isSaving,
	isDownloading,
	canSave,
	notice,
	onSave,
	onPreview,
	onDownload,
}: DoeHeaderProps) {
	return (
		<header className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
			<div className="min-w-0">
				<nav
					aria-label="Fil d'Ariane"
					className="mb-1 flex flex-wrap items-center gap-1 text-xs font-semibold uppercase text-brand-muted"
				>
					<Link to={buildOrgPath(organizationSlug, '/planification/projects')}>
						Projets
					</Link>
					<span>/</span>
					<Link
						to={buildOrgPath(
							organizationSlug,
							'/planification/projects/$projectId',
						)}
						params={{ projectId }}
					>
						{projectName}
					</Link>
					<span>/</span>
					<span>DOE</span>
				</nav>
				<h1 className="truncate text-2xl font-normal md:text-[28px]">
					DOE – {title}
				</h1>
				<div className="mt-2 flex flex-wrap items-center gap-2">
					<Badge variant="outline">Indice {revision}</Badge>
					{lot.trim() !== '' ? <Badge variant="secondary">{lot}</Badge> : null}
					{isDirty ? (
						<Badge variant="outline" className="text-warning">
							Modifications non enregistrées
						</Badge>
					) : null}
					{notice ? (
						<output className="text-xs text-success">{notice}</output>
					) : null}
				</div>
			</div>
			<div className="flex flex-wrap gap-2">
				<Button
					type="button"
					variant="outline"
					disabled={isDownloading}
					onClick={onPreview}
				>
					<Eye />
					Aperçu du PDF
				</Button>
				<Button
					type="button"
					variant="outline"
					disabled={isDownloading}
					onClick={onDownload}
				>
					{isDownloading ? <Loader2 className="animate-spin" /> : <Download />}
					Télécharger le PDF
				</Button>
				<RequirePermission permission="MANAGE_PLANNING">
					<Button
						type="button"
						disabled={!isDirty || !canSave || isSaving}
						onClick={onSave}
					>
						{isSaving ? <Loader2 className="animate-spin" /> : <Save />}
						Enregistrer
					</Button>
				</RequirePermission>
			</div>
		</header>
	)
}
