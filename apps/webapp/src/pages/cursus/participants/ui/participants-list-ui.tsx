import {
	createColumnHelper,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import {
	KIND_LABELS,
	type ParticipantKind,
	type ParticipantStatus,
	STATUS_LABELS,
} from '#/pages/cursus/participants/lib/participants'
import { FormError } from '#/pages/cursus/participants/ui/field-input'

export interface ParticipantRowVM {
	id: string
	name: string
	email: string
	kinds: ParticipantKind[] | null
	hostCompanyNames: string[] | null
	status: ParticipantStatus
}

export interface ParticipantsListUIProps {
	rows: ParticipantRowVM[]
	isLoading: boolean
	error: string | null
	kind: ParticipantKind | undefined
	status: ParticipantStatus | undefined
	canManage: boolean
	page: number
	pageCount: number
	onKindChange: (kind: ParticipantKind | undefined) => void
	onStatusChange: (status: ParticipantStatus | undefined) => void
	onPageChange: (page: number) => void
	onCreate: () => void
	onOpen: (participantId: string) => void
}

const ALL = 'all'
const columnHelper = createColumnHelper<ParticipantRowVM>()

export function ParticipantsListUI({
	rows,
	isLoading,
	error,
	kind,
	status,
	canManage,
	page,
	pageCount,
	onKindChange,
	onStatusChange,
	onPageChange,
	onCreate,
	onOpen,
}: ParticipantsListUIProps) {
	const columns = [
		columnHelper.accessor('name', {
			header: 'Nom',
			cell: (info) => (
				<button
					type="button"
					className="font-medium underline-offset-4 hover:underline"
					onClick={() => onOpen(info.row.original.id)}
				>
					{info.getValue()}
				</button>
			),
		}),
		columnHelper.accessor('email', { header: 'E-mail' }),
		columnHelper.accessor('kinds', {
			header: 'Type',
			cell: (info) => {
				const kinds = info.getValue()
				if (kinds === null) return '…'
				if (kinds.length === 0) return '—'
				return kinds.map((value) => KIND_LABELS[value]).join(', ')
			},
		}),
		columnHelper.accessor('hostCompanyNames', {
			header: 'Entreprise',
			cell: (info) => {
				const names = info.getValue()
				if (names === null) return '…'
				return names.length === 0 ? '—' : names.join(', ')
			},
		}),
		columnHelper.accessor('status', {
			header: 'Statut',
			cell: (info) => (
				<Badge variant={info.getValue() === 'active' ? 'default' : 'secondary'}>
					{STATUS_LABELS[info.getValue()]}
				</Badge>
			),
		}),
	]

	const table = useReactTable({
		data: rows,
		columns,
		getCoreRowModel: getCoreRowModel(),
	})

	return (
		<div className="flex flex-col gap-4 p-6">
			<div className="flex flex-wrap items-center justify-between gap-3">
				<h1 className="text-xl font-semibold">Participants</h1>
				{canManage ? (
					<Button type="button" onClick={onCreate}>
						Nouveau participant
					</Button>
				) : null}
			</div>

			<div className="flex flex-wrap gap-3">
				<Select
					value={kind ?? ALL}
					onValueChange={(value) =>
						onKindChange(value === ALL ? undefined : (value as ParticipantKind))
					}
				>
					<SelectTrigger aria-label="Type" className="w-52">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={ALL}>Tous les types</SelectItem>
						{(Object.keys(KIND_LABELS) as ParticipantKind[]).map((value) => (
							<SelectItem key={value} value={value}>
								{KIND_LABELS[value]}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
				<Select
					value={status ?? ALL}
					onValueChange={(value) =>
						onStatusChange(
							value === ALL ? undefined : (value as ParticipantStatus),
						)
					}
				>
					<SelectTrigger aria-label="Statut" className="w-48">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={ALL}>Tous les statuts</SelectItem>
						{(Object.keys(STATUS_LABELS) as ParticipantStatus[]).map(
							(value) => (
								<SelectItem key={value} value={value}>
									{STATUS_LABELS[value]}
								</SelectItem>
							),
						)}
					</SelectContent>
				</Select>
			</div>

			<FormError message={error} />

			<Table>
				<TableHeader>
					{table.getHeaderGroups().map((group) => (
						<TableRow key={group.id}>
							{group.headers.map((header) => (
								<TableHead key={header.id}>
									{flexRender(
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
						<TableRow key={row.id}>
							{row.getVisibleCells().map((cell) => (
								<TableCell key={cell.id}>
									{flexRender(cell.column.columnDef.cell, cell.getContext())}
								</TableCell>
							))}
						</TableRow>
					))}
				</TableBody>
			</Table>

			{isLoading ? (
				<p className="text-sm text-muted-foreground">Chargement…</p>
			) : null}
			{!isLoading && rows.length === 0 && !error ? (
				<p className="text-sm text-muted-foreground">Aucun participant.</p>
			) : null}

			{pageCount > 1 ? (
				<div className="flex items-center justify-end gap-3 text-sm">
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={page <= 1}
						onClick={() => onPageChange(page - 1)}
					>
						Précédent
					</Button>
					<span>
						Page {page} sur {pageCount}
					</span>
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={page >= pageCount}
						onClick={() => onPageChange(page + 1)}
					>
						Suivant
					</Button>
				</div>
			) : null}
		</div>
	)
}
