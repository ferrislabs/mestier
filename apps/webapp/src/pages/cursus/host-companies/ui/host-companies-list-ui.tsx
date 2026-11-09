import {
	createColumnHelper,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
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
import type { HostCompanyValues } from '#/pages/cursus/host-companies/lib/host-companies'
import { HostCompanyForm } from '#/pages/cursus/host-companies/ui/host-company-form'
import { FormError } from '#/pages/cursus/participants/ui/field-input'

export interface HostCompanyRowVM {
	id: string
	name: string
	siret: string
	nafCode: string
	city: string
	employeeCount: string
	archived: boolean
}

export interface HostCompaniesListUIProps {
	rows: HostCompanyRowVM[]
	isLoading: boolean
	error: string | null
	includeArchived: boolean
	canManage: boolean
	createOpen: boolean
	isCreating: boolean
	createError: string | null
	onIncludeArchivedChange: (includeArchived: boolean) => void
	onCreateOpenChange: (open: boolean) => void
	onCreate: (values: HostCompanyValues) => void
	onOpen: (hostCompanyId: string) => void
}

const columnHelper = createColumnHelper<HostCompanyRowVM>()

export function HostCompaniesListUI({
	rows,
	isLoading,
	error,
	includeArchived,
	canManage,
	createOpen,
	isCreating,
	createError,
	onIncludeArchivedChange,
	onCreateOpenChange,
	onCreate,
	onOpen,
}: HostCompaniesListUIProps) {
	const columns = [
		columnHelper.accessor('name', {
			header: "Nom de l'entreprise",
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
		columnHelper.accessor('siret', { header: 'Numéro de SIRET' }),
		columnHelper.accessor('nafCode', { header: 'NAF' }),
		columnHelper.accessor('city', { header: 'Ville' }),
		columnHelper.accessor('employeeCount', { header: 'Employés' }),
		columnHelper.accessor('archived', {
			header: 'Statut',
			cell: (info) =>
				info.getValue() ? (
					<Badge variant="secondary">Archivée</Badge>
				) : (
					'Active'
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
				<h1 className="text-xl font-semibold">Entreprises d'accueil</h1>
				{canManage ? (
					<Button type="button" onClick={() => onCreateOpenChange(true)}>
						Nouvelle entreprise
					</Button>
				) : null}
			</div>

			<Select
				value={includeArchived ? 'all' : 'active'}
				onValueChange={(value) => onIncludeArchivedChange(value === 'all')}
			>
				<SelectTrigger aria-label="Archivage" className="w-56">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value="active">Actives</SelectItem>
					<SelectItem value="all">Avec les archivées</SelectItem>
				</SelectContent>
			</Select>

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
				<p className="text-sm text-muted-foreground">Aucune entreprise.</p>
			) : null}

			{canManage ? (
				<Dialog open={createOpen} onOpenChange={onCreateOpenChange}>
					<DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
						<DialogHeader>
							<DialogTitle>Nouvelle entreprise d'accueil</DialogTitle>
							<DialogDescription>
								Renseignez l'entreprise qui accueille des apprentis.
							</DialogDescription>
						</DialogHeader>
						<HostCompanyForm
							submitLabel="Créer l'entreprise"
							isPending={isCreating}
							error={createError}
							onSubmit={onCreate}
						/>
					</DialogContent>
				</Dialog>
			) : null}
		</div>
	)
}
