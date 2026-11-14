import { Archive, ArchiveRestore, Pencil } from 'lucide-react'
import type { ReactNode } from 'react'
import { PageShell, StatusBadge } from '#/components/ui/surface'
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
	type ArchiveFilter,
	MASCULINE_ARCHIVE_OPTIONS,
} from '#/pages/cursus/shared/lib/list-filters'
import { ListEmptyRow } from '#/pages/cursus/shared/ui/list-empty'
import { ListPagination } from '#/pages/cursus/shared/ui/list-pagination'
import { ListToolbar } from '#/pages/cursus/shared/ui/list-toolbar'
import { RowActions } from '#/pages/cursus/shared/ui/row-actions'

export interface SchoolTutorRowVM {
	id: string
	name: string
	jobTitle: string | null
	openTriadCount: number
	archived: boolean
}

export interface SchoolTutorsListUIProps {
	header: ReactNode
	rows: SchoolTutorRowVM[]
	search: string
	archived: ArchiveFilter
	isLoading: boolean
	error: string | null
	canManage: boolean
	page: number
	pageCount: number
	onSearchChange: (search: string) => void
	onArchivedChange: (archived: ArchiveFilter) => void
	onResetFilters: () => void
	onPageChange: (page: number) => void
	onEditJobTitle: (id: string) => void
	onArchive: (id: string) => void
	onRestore: (id: string) => void
}

function rowActions(
	row: SchoolTutorRowVM,
	{
		onEditJobTitle,
		onArchive,
		onRestore,
	}: Pick<
		SchoolTutorsListUIProps,
		'onEditJobTitle' | 'onArchive' | 'onRestore'
	>,
) {
	if (row.archived) {
		return [
			{
				label: 'Restaurer',
				icon: <ArchiveRestore />,
				onSelect: () => onRestore(row.id),
			},
		]
	}
	return [
		{
			label: 'Modifier la fonction',
			icon: <Pencil />,
			onSelect: () => onEditJobTitle(row.id),
		},
		{
			label: 'Archiver',
			icon: <Archive />,
			onSelect: () => onArchive(row.id),
		},
	]
}

export function SchoolTutorsListUI({
	header,
	rows,
	search,
	archived,
	isLoading,
	error,
	canManage,
	page,
	pageCount,
	onSearchChange,
	onArchivedChange,
	onResetFilters,
	onPageChange,
	...actions
}: SchoolTutorsListUIProps) {
	const filtered = search.trim() !== '' || archived !== 'active'

	return (
		<PageShell className="block">
			{header}
			<ListToolbar
				search={search}
				searchPlaceholder="Rechercher un nom"
				onSearch={onSearchChange}
				status={archived}
				statusOptions={MASCULINE_ARCHIVE_OPTIONS}
				statusLabel="Filtrer par statut"
				onStatus={onArchivedChange}
			/>
			<FormError message={error} />
			<div className="overflow-hidden rounded-lg border bg-card">
				<Table>
					<TableHeader>
						<TableRow className="hover:bg-transparent">
							<TableHead className="pl-4">Nom</TableHead>
							<TableHead>Fonction</TableHead>
							<TableHead>Trinômes suivis</TableHead>
							<TableHead>Statut</TableHead>
							<TableHead className="w-12 pr-4">
								<span className="sr-only">Actions</span>
							</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{isLoading ? null : rows.length === 0 ? (
							<ListEmptyRow
								colSpan={5}
								title="Aucun tuteur école ne correspond."
								hint={
									filtered
										? 'Vérifie l’orthographe ou le statut sélectionné.'
										: undefined
								}
								onReset={filtered ? onResetFilters : undefined}
							/>
						) : (
							rows.map((row) => (
								<TableRow
									key={row.id}
									className={cn('group', row.archived && 'opacity-60')}
								>
									<TableCell className="py-2.5 pl-4 font-medium">
										{row.name}
									</TableCell>
									<TableCell className="text-muted-foreground">
										{row.jobTitle ?? '—'}
									</TableCell>
									<TableCell className="tabular">
										{row.openTriadCount}
									</TableCell>
									<TableCell>
										<StatusBadge tone={row.archived ? 'neutral' : 'success'}>
											{row.archived ? 'Archivé' : 'Actif'}
										</StatusBadge>
									</TableCell>
									<TableCell className="pr-4">
										<RowActions
											label={row.name}
											actions={canManage ? rowActions(row, actions) : []}
										/>
									</TableCell>
								</TableRow>
							))
						)}
					</TableBody>
				</Table>
			</div>
			<ListPagination
				page={page}
				pageCount={pageCount}
				onPageChange={onPageChange}
			/>
		</PageShell>
	)
}
