import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { ClosedBadge } from '#/pages/cursus/shared/ui/status-badge'
import type { TriadSummary } from '#/pages/cursus/triads/feature/use-triads'

interface TriadsTableProps {
	triads: TriadSummary[]
	showPromotion: boolean
	onOpen: (triadId: string) => void
}

export function TriadsTable({
	triads,
	showPromotion,
	onOpen,
}: TriadsTableProps) {
	const columns: ColumnDef<TriadSummary>[] = [
		{ header: 'Apprenti', accessorKey: 'apprentice_name' },
	]
	if (showPromotion) {
		columns.push({
			header: 'Promotion',
			cell: ({ row }) =>
				`${row.original.promotion_label} · ${row.original.training_name}`,
		})
	}
	columns.push(
		{
			header: 'Tuteur pédagogique',
			cell: ({ row }) => row.original.pedagogic_tutor_name ?? '—',
		},
		{
			header: 'Entreprise actuelle',
			cell: ({ row }) => row.original.current_host_company_name ?? '—',
		},
		{
			header: 'Statut',
			cell: ({ row }) => (
				<ClosedBadge closed={Boolean(row.original.closed_at)} />
			),
		},
	)

	const table = useReactTable({
		data: triads,
		columns,
		getCoreRowModel: getCoreRowModel(),
	})

	return (
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
									{flexRender(cell.column.columnDef.cell, cell.getContext())}
								</TableCell>
							))}
						</TableRow>
					))}
				</TableBody>
			</Table>
		</div>
	)
}
