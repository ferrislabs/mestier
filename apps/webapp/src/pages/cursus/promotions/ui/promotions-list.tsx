import { Archive, Pencil, Plus } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { PageHeader, PageShell, StatusBadge } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { cn } from '#/lib/utils'
import type { Promotion } from '#/pages/cursus/promotions/feature/use-promotions'
import {
	archiveQualifier,
	countSummary,
	PROMOTION_NOUN,
} from '#/pages/cursus/shared/lib/count-summary'
import {
	type ArchiveFilter,
	FEMININE_ARCHIVE_OPTIONS,
} from '#/pages/cursus/shared/lib/list-filters'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { ListEmpty } from '#/pages/cursus/shared/ui/list-empty'
import { ListToolbar } from '#/pages/cursus/shared/ui/list-toolbar'
import { RowActions } from '#/pages/cursus/shared/ui/row-actions'

const ALL_TRAININGS = 'all'

interface PromotionsListProps {
	promotions: Promotion[]
	total: number | undefined
	trainingNames: Record<string, string>
	trainings: SelectOption[]
	trainingId: string | undefined
	archived: ArchiveFilter
	search: string
	archiveError: string | null
	isLoading: boolean
	canManage: boolean
	onSearchChange: (search: string) => void
	onTrainingChange: (trainingId: string | undefined) => void
	onArchivedChange: (archived: ArchiveFilter) => void
	onResetFilters: () => void
	onOpen: (promotionId: string) => void
	onCreate: () => void
	onEdit: (promotion: Promotion) => void
	onArchive: (promotion: Promotion) => void
}

export function PromotionsList({
	promotions,
	total,
	trainingNames,
	trainings,
	trainingId,
	archived,
	search,
	archiveError,
	isLoading,
	canManage,
	onSearchChange,
	onTrainingChange,
	onArchivedChange,
	onResetFilters,
	onOpen,
	onCreate,
	onEdit,
	onArchive,
}: PromotionsListProps) {
	const filtered =
		search.trim() !== '' || archived !== 'active' || trainingId !== undefined

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title="Promotions"
				description={
					total === undefined
						? undefined
						: countSummary(total, PROMOTION_NOUN, archiveQualifier(archived))
				}
				actions={
					canManage ? (
						<Button type="button" size="sm" onClick={onCreate}>
							<Plus aria-hidden /> Nouvelle promotion
						</Button>
					) : null
				}
			/>
			<ListToolbar
				search={search}
				searchPlaceholder="Rechercher une promotion"
				onSearch={onSearchChange}
				status={archived}
				statusOptions={FEMININE_ARCHIVE_OPTIONS}
				statusLabel="Filtrer par statut"
				onStatus={onArchivedChange}
				select={{
					value: trainingId ?? ALL_TRAININGS,
					label: 'Filtrer par formation',
					options: [
						{ value: ALL_TRAININGS, label: 'Toutes les formations' },
						...trainings,
					],
					onChange: (next) =>
						onTrainingChange(next === ALL_TRAININGS ? undefined : next),
				}}
			/>
			{archiveError ? (
				<p role="alert" className="mb-3 text-[13px] text-state-alert">
					{archiveError}
				</p>
			) : null}
			{isLoading ? null : promotions.length === 0 ? (
				<ListEmpty
					title="Aucune promotion ne correspond."
					hint={
						filtered
							? 'Vérifie l’orthographe, le statut ou la formation sélectionnée.'
							: canManage
								? 'Créez une première promotion pour y rattacher des trinômes.'
								: undefined
					}
					onReset={filtered ? onResetFilters : undefined}
				/>
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Libellé</TableHead>
								<TableHead>Formation</TableHead>
								<TableHead>Années</TableHead>
								<TableHead>Statut</TableHead>
								<TableHead className="w-12 pr-4">
									<span className="sr-only">Actions</span>
								</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{promotions.map((promotion) => {
								const isArchived = Boolean(promotion.archived_at)
								return (
									<TableRow
										key={promotion.id}
										className={cn(
											'group cursor-pointer',
											isArchived && 'opacity-60',
										)}
										onClick={() => onOpen(promotion.id)}
									>
										<TableCell className="py-2.5 pl-4 font-medium">
											{promotion.label}
										</TableCell>
										<TableCell>
											{trainingNames[promotion.training_id] ?? '—'}
										</TableCell>
										<TableCell className="tabular">
											{`${promotion.start_year}–${promotion.end_year}`}
										</TableCell>
										<TableCell>
											<StatusBadge tone={isArchived ? 'neutral' : 'success'}>
												{isArchived ? 'Archivée' : 'Active'}
											</StatusBadge>
										</TableCell>
										<TableCell className="pr-4">
											<RowActions
												label={promotion.label}
												actions={
													canManage
														? [
																{
																	label: 'Modifier',
																	icon: <Pencil />,
																	onSelect: () => onEdit(promotion),
																},
																...(isArchived
																	? []
																	: [
																			{
																				label: 'Archiver',
																				icon: <Archive />,
																				onSelect: () => onArchive(promotion),
																			},
																		]),
															]
														: []
												}
											/>
										</TableCell>
									</TableRow>
								)
							})}
						</TableBody>
					</Table>
				</div>
			)}
		</PageShell>
	)
}
