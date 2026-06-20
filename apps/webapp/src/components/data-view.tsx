import {
	ArrowUpDown,
	ChevronLeft,
	ChevronRight,
	ChevronsLeft,
	ChevronsRight,
	Filter,
	Search,
	X,
} from 'lucide-react'
import { useMemo } from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuLabel,
	DropdownMenuRadioGroup,
	DropdownMenuRadioItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'

export interface DataViewOption {
	value: string
	label: string
}

export interface DataViewSortOption<TItem> extends DataViewOption {
	compare: (a: TItem, b: TItem) => number
}

interface UseDataViewOptions<TItem> {
	data: TItem[]
	search: string
	searchPredicate: (item: TItem, query: string) => boolean
	filter: string
	filterPredicate: (item: TItem, filter: string) => boolean
	sort: string
	sortOptions: DataViewSortOption<TItem>[]
	page: number
	pageSize: number
}

export function useDataView<TItem>({
	data,
	search,
	searchPredicate,
	filter,
	filterPredicate,
	sort,
	sortOptions,
	page,
	pageSize,
}: UseDataViewOptions<TItem>) {
	return useMemo(() => {
		const query = search.trim().toLowerCase()
		const activeSort =
			sortOptions.find((option) => option.value === sort) ?? sortOptions[0]
		const filtered = data.filter((item) => {
			const matchesSearch = query ? searchPredicate(item, query) : true
			const matchesFilter = filterPredicate(item, filter)
			return matchesSearch && matchesFilter
		})
		const sorted = activeSort
			? [...filtered].sort(activeSort.compare)
			: filtered
		const pageCount = Math.max(1, Math.ceil(sorted.length / pageSize))
		const currentPage = Math.min(Math.max(page, 1), pageCount)
		const fromIndex = (currentPage - 1) * pageSize
		const rows = sorted.slice(fromIndex, fromIndex + pageSize)

		return {
			rows,
			totalCount: data.length,
			filteredCount: sorted.length,
			page: currentPage,
			pageCount,
			from: sorted.length === 0 ? 0 : fromIndex + 1,
			to: Math.min(fromIndex + pageSize, sorted.length),
		}
	}, [
		data,
		filter,
		filterPredicate,
		page,
		pageSize,
		search,
		searchPredicate,
		sort,
		sortOptions,
	])
}

interface DataViewToolbarProps {
	search: string
	onSearchChange: (value: string) => void
	searchPlaceholder?: string
	filter: string
	onFilterChange: (value: string) => void
	filterOptions: DataViewOption[]
	sort: string
	onSortChange: (value: string) => void
	sortOptions: DataViewOption[]
	filteredCount: number
	totalCount: number
	onReset: () => void
}

export function DataViewToolbar({
	search,
	onSearchChange,
	searchPlaceholder = 'Rechercher…',
	filter,
	onFilterChange,
	filterOptions,
	sort,
	onSortChange,
	sortOptions,
	filteredCount,
	totalCount,
	onReset,
}: DataViewToolbarProps) {
	const hasActiveCriteria = search.trim() || filter !== 'all'
	const activeFilterLabel =
		filterOptions.find((option) => option.value === filter)?.label ?? 'Filtres'

	return (
		<div className="flex flex-col gap-3 rounded-lg border bg-card p-3 shadow-xs">
			<div className="flex flex-col gap-3 lg:flex-row lg:items-center">
				<div className="relative min-w-0 flex-1">
					<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
					<Input
						type="search"
						value={search}
						onChange={(event) => onSearchChange(event.target.value)}
						placeholder={searchPlaceholder}
						className="pl-9"
					/>
				</div>

				<div className="grid grid-cols-1 gap-2 sm:grid-cols-[auto_minmax(0,14rem)_auto]">
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button
								type="button"
								variant={filter === 'all' ? 'outline' : 'default'}
								className="justify-start"
							>
								<Filter />
								Filtres
								{filter !== 'all' ? (
									<span className="rounded-md bg-primary-foreground/20 px-1.5 py-0.5 text-[11px]">
										1
									</span>
								) : null}
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end" className="w-64">
							<DropdownMenuLabel>Filtrer par</DropdownMenuLabel>
							<DropdownMenuSeparator />
							<DropdownMenuRadioGroup
								value={filter}
								onValueChange={onFilterChange}
							>
								{filterOptions.map((option) => (
									<DropdownMenuRadioItem
										key={option.value}
										value={option.value}
									>
										{option.label}
									</DropdownMenuRadioItem>
								))}
							</DropdownMenuRadioGroup>
						</DropdownMenuContent>
					</DropdownMenu>

					<Select value={sort} onValueChange={onSortChange}>
						<SelectTrigger className="w-full">
							<ArrowUpDown className="size-4" />
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{sortOptions.map((option) => (
								<SelectItem key={option.value} value={option.value}>
									{option.label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>

					<Button
						type="button"
						variant="ghost"
						disabled={!hasActiveCriteria}
						onClick={onReset}
					>
						<X />
						Réinitialiser
					</Button>
				</div>
			</div>

			<p className="text-xs text-muted-foreground">
				{filteredCount} résultat{filteredCount > 1 ? 's' : ''} sur {totalCount}
				{filter !== 'all' ? ` · ${activeFilterLabel}` : null}
			</p>
		</div>
	)
}

interface DataViewPaginationProps {
	page: number
	pageCount: number
	pageSize: number
	pageSizeOptions?: number[]
	from: number
	to: number
	totalCount: number
	onPageChange: (page: number) => void
	onPageSizeChange: (pageSize: number) => void
}

export function DataViewPagination({
	page,
	pageCount,
	pageSize,
	pageSizeOptions = [10, 25, 50],
	from,
	to,
	totalCount,
	onPageChange,
	onPageSizeChange,
}: DataViewPaginationProps) {
	const rangeLabel =
		totalCount === 0 ? '0 sur 0' : `${from}-${to} sur ${totalCount}`

	return (
		<div className="flex flex-col gap-3 rounded-lg border bg-card px-3 py-3 text-sm text-muted-foreground shadow-xs sm:flex-row sm:items-center sm:justify-between">
			<p>{rangeLabel}</p>

			<div className="flex flex-wrap items-center gap-2">
				<Select
					value={String(pageSize)}
					onValueChange={(value) => onPageSizeChange(Number(value))}
				>
					<SelectTrigger className="w-28">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{pageSizeOptions.map((option) => (
							<SelectItem key={option} value={String(option)}>
								{option} / page
							</SelectItem>
						))}
					</SelectContent>
				</Select>

				<div className="flex items-center gap-1">
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={page <= 1}
						onClick={() => onPageChange(1)}
					>
						<ChevronsLeft />
						<span className="sr-only">Première page</span>
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={page <= 1}
						onClick={() => onPageChange(page - 1)}
					>
						<ChevronLeft />
						<span className="sr-only">Page précédente</span>
					</Button>
					<span className="min-w-20 text-center text-xs font-medium text-foreground">
						{page} / {pageCount}
					</span>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={page >= pageCount}
						onClick={() => onPageChange(page + 1)}
					>
						<ChevronRight />
						<span className="sr-only">Page suivante</span>
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						disabled={page >= pageCount}
						onClick={() => onPageChange(pageCount)}
					>
						<ChevronsRight />
						<span className="sr-only">Dernière page</span>
					</Button>
				</div>
			</div>
		</div>
	)
}
