import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import { AlertCircle, LifeBuoy } from 'lucide-react'
import { useMemo } from 'react'
import { Button } from '#/components/ui/button'
import {
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import { supportStatusTone } from '#/pages/support/lib/columns'
import type { SupportRowVM, SupportTableUIProps } from '#/pages/support/types'

function buildColumns(fieldKeys: string[]): ColumnDef<SupportRowVM>[] {
	return [
		{
			id: 'number',
			header: 'N°',
			cell: ({ row }) => row.original.numberLabel,
		},
		{
			id: 'subject',
			header: 'Sujet',
			cell: ({ row }) => row.original.subject,
		},
		{
			id: 'status',
			header: 'Statut',
			cell: ({ row }) => (
				<StatusBadge tone={supportStatusTone(row.original.status)}>
					{row.original.statusLabel}
				</StatusBadge>
			),
		},
		{
			id: 'assignee',
			header: 'Assigné à',
			cell: ({ row }) => row.original.assigneeLabel,
		},
		...fieldKeys.map(
			(fieldKey): ColumnDef<SupportRowVM> => ({
				id: `field:${fieldKey}`,
				header: fieldKey,
				cell: ({ row }) => row.original.fields[fieldKey] ?? '',
			}),
		),
		{
			id: 'badges',
			header: 'Badges',
			cell: ({ row }) => (
				<div className="flex flex-wrap gap-1">
					{row.original.badges.map((badge) => (
						<StatusBadge key={badge.label} tone={badge.tone}>
							{badge.label}
						</StatusBadge>
					))}
				</div>
			),
		},
	]
}

export function SupportTableUI({
	organizationName,
	rows,
	fieldKeys,
	isLoading,
	error,
	onRetry,
	toolbar,
	emptyReason,
	truncationNotice,
}: SupportTableUIProps) {
	const columns = useMemo(() => buildColumns(fieldKeys), [fieldKeys])
	const table = useReactTable({
		data: rows,
		columns,
		getCoreRowModel: getCoreRowModel(),
		getRowId: (row) => row.id,
	})
	const isEmpty = !isLoading && !error && rows.length === 0

	return (
		<PageShell className="max-w-none overflow-x-hidden">
			<PageHeader
				title="Support"
				description="Tous les tickets en liste, avec leurs champs personnalisés."
				eyebrow={organizationName}
				className="sm:items-center sm:justify-start"
				leading={
					<div className="flex size-12 items-center justify-center rounded-lg bg-brand-soft text-primary">
						<LifeBuoy className="size-6" />
					</div>
				}
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					<Button onClick={onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</SectionCard>
			) : null}

			<SectionCard className="flex min-w-0 flex-col gap-4 overflow-hidden p-4">
				<div className="flex flex-wrap items-center gap-2">
					<h2 className="font-semibold">Tickets</h2>
					<StatusBadge tone="neutral">
						{rows.length} ticket{rows.length > 1 ? 's' : ''}
					</StatusBadge>
				</div>

				{toolbar}

				{truncationNotice ? (
					<output className="block rounded-md border border-dashed bg-muted/25 px-3 py-2 text-sm text-muted-foreground">
						{truncationNotice}
					</output>
				) : null}

				{isEmpty ? (
					<div className="flex flex-col items-center justify-center gap-2 rounded-md border border-dashed bg-muted/25 p-10 text-center">
						<p className="text-sm font-semibold">Aucun ticket</p>
						<p className="text-sm text-muted-foreground">
							{emptyReason ?? 'Aucun ticket pour le moment.'}
						</p>
					</div>
				) : (
					<div className="min-w-0 max-w-full overflow-x-auto overscroll-x-contain">
						<table className="w-full min-w-[720px] border-collapse text-sm">
							<thead>
								{table.getHeaderGroups().map((headerGroup) => (
									<tr key={headerGroup.id} className="border-b bg-muted/50">
										{headerGroup.headers.map((header) => (
											<th
												key={header.id}
												className="px-4 py-3 text-left text-xs font-semibold uppercase text-muted-foreground"
											>
												{header.isPlaceholder
													? null
													: flexRender(
															header.column.columnDef.header,
															header.getContext(),
														)}
											</th>
										))}
									</tr>
								))}
							</thead>
							<tbody>
								{isLoading ? (
									<tr>
										<td
											colSpan={columns.length}
											className="px-4 py-10 text-center text-xs text-muted-foreground"
										>
											Chargement…
										</td>
									</tr>
								) : rows.length === 0 ? (
									<tr>
										<td
											colSpan={columns.length}
											className="px-4 py-10 text-center text-xs text-muted-foreground"
										>
											Aucun ticket
										</td>
									</tr>
								) : (
									table.getRowModel().rows.map((row) => (
										<tr key={row.id} className="border-b last:border-b-0">
											{row.getVisibleCells().map((cell) => (
												<td key={cell.id} className="px-4 py-3 align-middle">
													{flexRender(
														cell.column.columnDef.cell,
														cell.getContext(),
													)}
												</td>
											))}
										</tr>
									))
								)}
							</tbody>
						</table>
					</div>
				)}
			</SectionCard>
		</PageShell>
	)
}
