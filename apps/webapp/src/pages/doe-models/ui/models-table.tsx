import { Link } from '@tanstack/react-router'
import { FilePlus } from 'lucide-react'
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
import type { DoeModel } from '#/hooks/use-doe-models'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import {
	isArchived,
	themeCounts,
	themesLabel,
} from '#/pages/doe-models/lib/models-filter'
import { StatusBadge } from '#/pages/projects/ui/badges'

export function ModelsTable({
	organizationSlug,
	models,
	onArchive,
	onRestore,
	onCreateDoe,
}: {
	organizationSlug: string
	models: readonly DoeModel[]
	onArchive: (model: DoeModel) => void
	onRestore: (model: DoeModel) => void
	onCreateDoe: (model: DoeModel) => void
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Modèle</TableHead>
						<TableHead className="max-md:hidden">Thèmes</TableHead>
						<TableHead className="pr-4 text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{models.map((model) => {
						const archived = isArchived(model)
						return (
							<TableRow
								key={model.id}
								className="relative cursor-pointer align-top focus-within:bg-muted/50"
							>
								<TableCell className="max-w-96 whitespace-normal py-3 pl-4">
									<div className="flex flex-wrap items-center gap-2">
										<Link
											to={buildOrgPath(
												organizationSlug,
												'/planification/doe-models/$modelId',
											)}
											params={{ modelId: model.id }}
											className={cn(
												'font-medium outline-none before:absolute before:inset-0 hover:underline focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset',
												archived && 'opacity-70',
											)}
										>
											{model.name}
										</Link>
										{archived && (
											<StatusBadge tone="neutral">Archivé</StatusBadge>
										)}
									</div>
									{model.description ? (
										<p className="mt-0.5 line-clamp-2 text-[13px] text-muted-foreground">
											{model.description}
										</p>
									) : null}
								</TableCell>
								<TableCell className="py-3 text-[13px] max-md:hidden">
									{themesLabel(themeCounts(model))}
								</TableCell>
								<TableCell className="py-2 pr-4">
									<RequirePermission permission="MANAGE_REFERENCE">
										<div className="relative z-10 flex flex-wrap items-center justify-end gap-1">
											{archived ? (
												<Button
													type="button"
													variant="outline"
													size="sm"
													aria-label={`Restaurer ${model.name}`}
													onClick={() => onRestore(model)}
												>
													Restaurer
												</Button>
											) : (
												<>
													<Button
														type="button"
														variant="outline"
														size="sm"
														aria-label={`Créer un DOE avec ce modèle (${model.name})`}
														onClick={() => onCreateDoe(model)}
													>
														<FilePlus aria-hidden />
														Créer un DOE avec ce modèle
													</Button>
													<Button
														type="button"
														variant="ghost"
														size="sm"
														aria-label={`Archiver ${model.name}`}
														onClick={() => onArchive(model)}
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
	)
}
