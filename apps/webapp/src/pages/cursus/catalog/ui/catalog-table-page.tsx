import { Archive, Pencil, Plus } from 'lucide-react'
import type { ReactNode } from 'react'
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
import { FormError } from '#/pages/cursus/participants/ui/field-input'
import {
	archiveQualifier,
	type CountNoun,
	countSummary,
} from '#/pages/cursus/shared/lib/count-summary'
import {
	type ArchiveChoice,
	type ArchiveFilter,
	archiveChoices,
	archiveFromChoices,
	type FilterOption,
} from '#/pages/cursus/shared/lib/list-filters'
import { ListEmptyRow } from '#/pages/cursus/shared/ui/list-empty'
import { ListToolbar } from '#/pages/cursus/shared/ui/list-toolbar'
import { RowActions } from '#/pages/cursus/shared/ui/row-actions'

export interface CatalogItem {
	id: string
	name: string
	archived_at?: string | null
}

export interface CatalogColumn<Item> {
	header: string
	cell: (item: Item) => ReactNode
}

export interface CatalogListState {
	total: number | undefined
	search: string
	archived: ArchiveFilter
	isLoading: boolean
	error: string | null
	canManage: boolean
	onSearchChange: (search: string) => void
	onArchivedChange: (archived: ArchiveFilter) => void
	onResetFilters: () => void
}

export interface CatalogTablePageProps<Item extends CatalogItem>
	extends CatalogListState {
	title: string
	addLabel: string
	noun: CountNoun
	searchPlaceholder: string
	archiveOptions: readonly FilterOption<ArchiveChoice>[]
	emptyTitle: string
	archivedLabel: string
	activeLabel: string
	columns: readonly CatalogColumn<Item>[]
	items: readonly Item[]
	onAdd: () => void
	onEdit: (id: string) => void
	onArchive: (id: string) => void
	dialog: ReactNode
}

export function CatalogTablePage<Item extends CatalogItem>({
	title,
	addLabel,
	noun,
	searchPlaceholder,
	archiveOptions,
	emptyTitle,
	archivedLabel,
	activeLabel,
	columns,
	items,
	total,
	search,
	archived,
	isLoading,
	error,
	canManage,
	onSearchChange,
	onArchivedChange,
	onResetFilters,
	onAdd,
	onEdit,
	onArchive,
	dialog,
}: CatalogTablePageProps<Item>) {
	const filtered = search.trim() !== '' || archived !== 'active'

	return (
		<PageShell className="block">
			<PageHeader
				className="mb-6"
				title={title}
				description={
					total === undefined
						? undefined
						: countSummary(total, noun, archiveQualifier(archived))
				}
				actions={
					canManage ? (
						<Button type="button" size="sm" onClick={onAdd}>
							<Plus aria-hidden /> {addLabel}
						</Button>
					) : null
				}
			/>
			<ListToolbar
				search={search}
				searchPlaceholder={searchPlaceholder}
				onSearch={onSearchChange}
				status={archiveChoices(archived)}
				statusOptions={archiveOptions}
				statusLabel="Statut"
				onStatus={(choices) => onArchivedChange(archiveFromChoices(choices))}
			/>
			<FormError message={error} />
			<div className="overflow-hidden rounded-lg border bg-card">
				<Table>
					<TableHeader>
						<TableRow className="hover:bg-transparent">
							<TableHead className="pl-4">Nom</TableHead>
							{columns.map((column) => (
								<TableHead key={column.header}>{column.header}</TableHead>
							))}
							<TableHead>Statut</TableHead>
							<TableHead className="w-12 pr-4">
								<span className="sr-only">Actions</span>
							</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{isLoading ? null : items.length === 0 ? (
							<ListEmptyRow
								colSpan={columns.length + 3}
								title={emptyTitle}
								hint={
									filtered
										? 'Vérifie l’orthographe ou le statut sélectionné.'
										: undefined
								}
								onReset={filtered ? onResetFilters : undefined}
							/>
						) : (
							items.map((item) => {
								const isArchived = Boolean(item.archived_at)
								return (
									<TableRow
										key={item.id}
										className={cn('group', isArchived && 'opacity-60')}
									>
										<TableCell className="py-2.5 pl-4 font-medium">
											{item.name}
										</TableCell>
										{columns.map((column) => (
											<TableCell key={column.header}>
												{column.cell(item)}
											</TableCell>
										))}
										<TableCell>
											<StatusBadge tone={isArchived ? 'neutral' : 'success'}>
												{isArchived ? archivedLabel : activeLabel}
											</StatusBadge>
										</TableCell>
										<TableCell className="pr-4">
											<RowActions
												label={item.name}
												actions={
													canManage && !isArchived
														? [
																{
																	label: 'Modifier',
																	icon: <Pencil />,
																	onSelect: () => onEdit(item.id),
																},
																{
																	label: 'Archiver',
																	icon: <Archive />,
																	onSelect: () => onArchive(item.id),
																},
															]
														: []
												}
											/>
										</TableCell>
									</TableRow>
								)
							})
						)}
					</TableBody>
				</Table>
			</div>
			{dialog}
		</PageShell>
	)
}
