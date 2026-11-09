import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import { Archive, Pencil, Plus } from 'lucide-react'
import { EmptyState } from '#/components/empty-state'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { Switch } from '#/components/ui/switch'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import type { Promotion } from '#/pages/cursus/promotions/feature/use-promotions'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'

interface PromotionsListProps {
	promotions: Promotion[]
	trainingNames: Record<string, string>
	trainings: SelectOption[]
	trainingId: string | undefined
	archived: boolean
	archiveError: string | null
	isLoading: boolean
	canManage: boolean
	onTrainingChange: (trainingId: string | undefined) => void
	onArchivedChange: (archived: boolean) => void
	onOpen: (promotionId: string) => void
	onCreate: () => void
	onEdit: (promotion: Promotion) => void
	onArchive: (promotion: Promotion) => void
}

export function PromotionsList({
	promotions,
	trainingNames,
	trainings,
	trainingId,
	archived,
	archiveError,
	isLoading,
	canManage,
	onTrainingChange,
	onArchivedChange,
	onOpen,
	onCreate,
	onEdit,
	onArchive,
}: PromotionsListProps) {
	const columns: ColumnDef<Promotion>[] = [
		{ header: 'Libellé', accessorKey: 'label' },
		{
			header: 'Formation',
			cell: ({ row }) => trainingNames[row.original.training_id] ?? '—',
		},
		{
			header: 'Années',
			cell: ({ row }) => `${row.original.start_year}–${row.original.end_year}`,
		},
		{
			header: 'Statut',
			cell: ({ row }) =>
				row.original.archived_at ? (
					<Badge variant="secondary">Archivée</Badge>
				) : (
					<Badge variant="outline">Active</Badge>
				),
		},
	]
	if (canManage) {
		columns.push({
			id: 'actions',
			header: '',
			cell: ({ row }) => (
				<div className="flex justify-end gap-1">
					<Button
						type="button"
						variant="ghost"
						size="sm"
						onClick={(event) => {
							event.stopPropagation()
							onEdit(row.original)
						}}
					>
						<Pencil />
						Modifier
					</Button>
					{row.original.archived_at ? null : (
						<Button
							type="button"
							variant="ghost"
							size="sm"
							onClick={(event) => {
								event.stopPropagation()
								onArchive(row.original)
							}}
						>
							<Archive />
							Archiver
						</Button>
					)}
				</div>
			),
		})
	}

	const table = useReactTable({
		data: promotions,
		columns,
		getCoreRowModel: getCoreRowModel(),
	})

	return (
		<PageShell>
			<PageHeader
				title="Promotions"
				description="Les promotions de chaque formation et leurs trinômes."
				actions={
					canManage ? (
						<Button type="button" onClick={onCreate}>
							<Plus />
							Nouvelle promotion
						</Button>
					) : null
				}
			/>
			<div className="flex flex-wrap items-center gap-4">
				<div className="w-64">
					<OptionSelect
						id="promotions-training-filter"
						value={trainingId ?? null}
						options={trainings}
						placeholder="Toutes les formations"
						noneLabel="Toutes les formations"
						onChange={(next) => onTrainingChange(next ?? undefined)}
					/>
				</div>
				<div className="flex items-center gap-2">
					<Switch
						id="promotions-archived"
						checked={archived}
						onCheckedChange={onArchivedChange}
					/>
					<Label htmlFor="promotions-archived">Archivées</Label>
				</div>
			</div>
			{archiveError ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{archiveError}
				</p>
			) : null}
			{!isLoading && promotions.length === 0 ? (
				<EmptyState
					title={archived ? 'Aucune promotion archivée' : 'Aucune promotion'}
					hint={
						canManage && !archived
							? 'Créez une première promotion pour y rattacher des trinômes.'
							: undefined
					}
				/>
			) : (
				<div className="rounded-lg border bg-card">
					<Table>
						<TableHeader>
							{table.getHeaderGroups().map((group) => (
								<TableRow key={group.id}>
									{group.headers.map((header) => (
										<TableHead key={header.id}>
											{header.isPlaceholder
												? null
												: flexRender(
														header.column.columnDef.header,
														header.getContext(),
													)}
										</TableHead>
									))}
								</TableRow>
							))}
						</TableHeader>
						<TableBody>
							{table.getRowModel().rows.map((row) => (
								<TableRow
									key={row.id}
									className="cursor-pointer"
									onClick={() => onOpen(row.original.id)}
								>
									{row.getVisibleCells().map((cell) => (
										<TableCell key={cell.id}>
											{flexRender(
												cell.column.columnDef.cell,
												cell.getContext(),
											)}
										</TableCell>
									))}
								</TableRow>
							))}
						</TableBody>
					</Table>
				</div>
			)}
		</PageShell>
	)
}
