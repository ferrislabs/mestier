import { Link } from '@tanstack/react-router'
import {
	ArrowLeft,
	Download,
	Eye,
	FileText,
	Images,
	Info,
	ListOrdered,
	Loader2,
	type LucideIcon,
	Package,
	Ruler,
	Save,
	Trash2,
} from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueNav } from '#/components/rubrique-layout'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import {
	type Rubrique,
	type RubriqueKind,
	tabSearch,
} from '#/pages/doe/lib/rubriques'

const ICONS: Record<RubriqueKind, LucideIcon> = {
	informations: Info,
	plan: ListOrdered,
	supplies: Package,
	photos: Images,
	plans: Ruler,
	blank_pages: FileText,
}

export interface DoeSidebarProps {
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
	errors: readonly string[]
	items: readonly Rubrique[]
	selected: string
	onSave: () => void
	onDownload: () => void
	onDownloadWord: () => void
	onDelete: () => void
}

export function DoeSidebar({
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
	errors,
	items,
	selected,
	onSave,
	onDownload,
	onDownloadWord,
	onDelete,
}: DoeSidebarProps) {
	return (
		<aside
			aria-label="DOE du projet"
			className="flex flex-col border-border border-b bg-card lg:min-h-0 lg:border-r lg:border-b-0"
		>
			<div className="shrink-0 border-border border-b px-4 py-4">
				<Link
					to={buildOrgPath(organizationSlug, '/projet/$projectId')}
					params={{ projectId }}
					aria-label={`Retour à l’aperçu du projet ${projectName}`}
					className="-ml-1 inline-flex max-w-full items-center gap-1.5 rounded-sm px-1 text-[13px] text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
				>
					<ArrowLeft className="size-4 shrink-0" aria-hidden />
					<span className="truncate">{projectName}</span>
				</Link>
				<h1 className="mt-3 break-words font-semibold text-lg leading-tight tracking-tight">
					DOE – {title}
				</h1>
				<div className="mt-1.5 flex flex-wrap items-center gap-1.5">
					<Badge variant="outline">Indice {revision}</Badge>
					{lot.trim() !== '' && <Badge variant="secondary">{lot}</Badge>}
					{isDirty && (
						<Badge variant="outline" className="text-state-today">
							Modifications non enregistrées
						</Badge>
					)}
				</div>
				<div className="mt-3 grid gap-2">
					<RequirePermission permission="MANAGE_PLANNING">
						<Button
							type="button"
							size="sm"
							disabled={!isDirty || !canSave || isSaving}
							onClick={onSave}
						>
							{isSaving ? (
								<Loader2 className="animate-spin" aria-hidden />
							) : (
								<Save aria-hidden />
							)}
							Enregistrer
						</Button>
					</RequirePermission>
					<div className="grid grid-cols-2 gap-2">
						<Button variant="outline" size="sm" asChild>
							<Link
								to={buildOrgPath(
									organizationSlug,
									'/projet/$projectId/doe/preview',
								)}
								params={{ projectId }}
							>
								<Eye aria-hidden />
								Aperçu du PDF
							</Link>
						</Button>
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={isDownloading}
							onClick={onDownload}
						>
							{isDownloading ? (
								<Loader2 className="animate-spin" aria-hidden />
							) : (
								<Download aria-hidden />
							)}
							Télécharger le PDF
						</Button>
					</div>
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={isDownloading}
						onClick={onDownloadWord}
					>
						{isDownloading ? (
							<Loader2 className="animate-spin" aria-hidden />
						) : (
							<FileText aria-hidden />
						)}
						Télécharger en Word
					</Button>
					<RequirePermission permission="MANAGE_PLANNING">
						<Button
							type="button"
							variant="ghost"
							size="sm"
							className="justify-start text-state-alert hover:bg-state-alert/10 hover:text-state-alert"
							onClick={onDelete}
						>
							<Trash2 aria-hidden />
							Supprimer le DOE
						</Button>
					</RequirePermission>
				</div>
				<output
					className={
						notice ? 'mt-2 block text-state-progress text-xs' : 'sr-only'
					}
				>
					{notice}
				</output>
				{errors.length > 0 && (
					<ul
						aria-label="Erreurs à corriger avant d’enregistrer"
						className="mt-3 grid gap-1 rounded-md border border-state-alert/30 bg-state-alert/8 px-3 py-2 text-[13px] text-state-alert"
					>
						{errors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
			</div>
			<RubriqueNav
				label="Rubriques du DOE"
				selected={selected}
				items={items.map((item) => ({
					id: item.id,
					label: item.label,
					summary: item.summary,
					icon: ICONS[item.kind],
				}))}
				renderLink={(item, props) => (
					<Link
						to={buildOrgPath(organizationSlug, '/projet/$projectId/doe')}
						params={{ projectId }}
						search={tabSearch(item.id)}
						{...props}
					/>
				)}
			/>
		</aside>
	)
}
