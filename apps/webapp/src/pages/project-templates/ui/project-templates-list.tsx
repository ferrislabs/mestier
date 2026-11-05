import { Loader2, Pencil, Plus } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import type { ProjectTemplate } from '#/hooks/use-project-templates'
import { cn } from '#/lib/utils'
import { StatusBadge } from '#/pages/projects/ui/badges'

export interface ProjectTemplatesListProps {
	templates: ProjectTemplate[]
	includeArchived: boolean
	isLoading: boolean
	error: string | null
	onIncludeArchivedChange: (includeArchived: boolean) => void
	onCreate: () => void
	onEdit: (template: ProjectTemplate) => void
	onArchive: (template: ProjectTemplate) => void
	onRestore: (template: ProjectTemplate) => void
}

export function ProjectTemplatesList({
	templates,
	includeArchived,
	isLoading,
	error,
	onIncludeArchivedChange,
	onCreate,
	onEdit,
	onArchive,
	onRestore,
}: ProjectTemplatesListProps) {
	return (
		<div>
			<header className="mb-4 flex flex-wrap items-end justify-between gap-4">
				<div className="min-w-0">
					<h2 className="font-semibold text-lg tracking-tight">
						Modèles de projet
					</h2>
					<p className="mt-1 max-w-2xl text-muted-foreground text-sm">
						Un modèle est un ensemble de tâches types avec des décalages, pas un
						projet à copier. Instancier un modèle produit un vrai projet, avec
						de vraies tâches, sur une date choisie.
					</p>
				</div>
				<RequirePermission permission="MANAGE_PLANNING">
					<Button onClick={onCreate}>
						<Plus aria-hidden />
						Nouveau modèle
					</Button>
				</RequirePermission>
			</header>

			{error ? (
				<div
					role="alert"
					className="mb-3 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
				>
					{error}
				</div>
			) : null}

			<div className="mb-3 flex flex-wrap items-center justify-between gap-3">
				<label
					htmlFor="include-archived-templates"
					className="flex items-center gap-2 text-[13px]"
				>
					<input
						id="include-archived-templates"
						type="checkbox"
						className="size-4 accent-primary"
						checked={includeArchived}
						onChange={(event) => onIncludeArchivedChange(event.target.checked)}
					/>
					Afficher les modèles archivés
				</label>
				<p className="text-[13px] text-muted-foreground">
					Archiver un modèle le retire du choix sans effacer ce qu’il a déjà
					produit.
				</p>
			</div>

			{isLoading ? (
				<div className="flex min-h-48 items-center justify-center gap-3 rounded-lg border bg-card p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" aria-hidden />
					Chargement des modèles…
				</div>
			) : templates.length === 0 ? (
				<EmptyState
					title="Aucun modèle de projet"
					hint="Crée-en un pour arrêter de reconstruire la même liste de tâches à chaque chantier."
				/>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Modèle</TableHead>
								<TableHead className="max-md:hidden">Description</TableHead>
								<TableHead className="pr-4 text-right">Actions</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{templates.map((template) => {
								const archived = Boolean(template.archived_at)
								return (
									<TableRow key={template.id} className="align-top">
										<TableCell className="max-w-96 whitespace-normal py-3 pl-4">
											<div className="flex flex-wrap items-center gap-2">
												<span
													className={cn(
														'font-medium',
														archived && 'opacity-70',
													)}
												>
													{template.name}
												</span>
												{archived && (
													<StatusBadge tone="neutral">Archivé</StatusBadge>
												)}
											</div>
										</TableCell>
										<TableCell className="max-w-96 py-3 text-[13px] max-md:hidden">
											{template.description ? (
												<span className="line-clamp-2 whitespace-normal">
													{template.description}
												</span>
											) : (
												<span className="text-muted-foreground">Aucune</span>
											)}
										</TableCell>
										<TableCell className="py-2 pr-4">
											<RequirePermission permission="MANAGE_PLANNING">
												<div className="flex flex-wrap items-center justify-end gap-1">
													{archived ? (
														<Button
															type="button"
															variant="outline"
															size="sm"
															aria-label={`Restaurer ${template.name}`}
															onClick={() => onRestore(template)}
														>
															Restaurer
														</Button>
													) : (
														<>
															<Button
																type="button"
																variant="outline"
																size="sm"
																aria-label={`Modifier ${template.name}`}
																onClick={() => onEdit(template)}
															>
																<Pencil aria-hidden />
																Modifier
															</Button>
															<Button
																type="button"
																variant="ghost"
																size="sm"
																aria-label={`Archiver ${template.name}`}
																onClick={() => onArchive(template)}
															>
																Archiver
															</Button>
														</>
													)}
												</div>
											</RequirePermission>
										</TableCell>
									</TableRow>
								)
							})}
						</TableBody>
					</Table>
				</div>
			)}
		</div>
	)
}
