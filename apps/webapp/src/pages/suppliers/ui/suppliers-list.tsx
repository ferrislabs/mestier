import type { ColumnDef } from '@tanstack/react-table'
import { Loader2, Plus, Truck } from 'lucide-react'
import { useMemo } from 'react'
import { ReferenceTable, RowIdentity } from '#/components/reference-table'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
} from '#/components/ui/surface'
import type { Supplier } from '#/hooks/use-suppliers'
import { firstAddressLine } from '#/pages/suppliers/types'

export interface SuppliersListProps {
	organizationName: string
	suppliers: Supplier[]
	isLoading: boolean
	error: string | null
	canManage: boolean
	onCreate: () => void
	onEdit: (supplier: Supplier) => void
}

function Dash({ value }: { value: string }) {
	return value ? (
		<span className="truncate">{value}</span>
	) : (
		<span className="text-muted-foreground">-</span>
	)
}

export function SuppliersList({
	organizationName,
	suppliers,
	isLoading,
	error,
	canManage,
	onCreate,
	onEdit,
}: SuppliersListProps) {
	const columns = useMemo<ColumnDef<Supplier>[]>(() => {
		const base: ColumnDef<Supplier>[] = [
			{
				id: 'name',
				header: 'Nom',
				cell: ({ row }) => (
					<RowIdentity title={row.original.name} id={row.original.id} />
				),
			},
			{
				id: 'address',
				header: 'Adresse',
				cell: ({ row }) => (
					<Dash value={firstAddressLine(row.original.address)} />
				),
			},
			{
				id: 'phone',
				header: 'Téléphone',
				cell: ({ row }) => <Dash value={row.original.phone ?? ''} />,
			},
			{
				id: 'email',
				header: 'Email',
				cell: ({ row }) => <Dash value={row.original.email ?? ''} />,
			},
		]
		if (!canManage) return base
		return [
			...base,
			{
				id: 'actions',
				header: () => <span className="sr-only">Actions</span>,
				cell: ({ row }) => (
					<div className="flex justify-end">
						<Button
							variant="outline"
							size="sm"
							onClick={() => onEdit(row.original)}
						>
							Modifier
						</Button>
					</div>
				),
			},
		]
	}, [canManage, onEdit])

	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Fournisseurs"
				description="Les fournisseurs de vos produits, rattachés aux fiches techniques du catalogue et repris dans le DOE."
				actions={
					<RequirePermission permission="MANAGE_REFERENCE">
						<Button onClick={onCreate}>
							<Plus />
							Nouveau fournisseur
						</Button>
					</RequirePermission>
				}
			/>

			<section className="grid grid-cols-2 gap-4 lg:grid-cols-3">
				<MetricCard
					label="Fournisseurs"
					value={suppliers.length}
					icon={<Truck className="size-4" />}
				/>
			</section>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement des fournisseurs…
				</SectionCard>
			) : (
				<ReferenceTable
					title={`Fournisseurs (${suppliers.length})`}
					description="Triés par nom."
					emptyTitle="Aucun fournisseur"
					emptyDescription="Ajoutez vos fournisseurs pour les rattacher aux produits du catalogue."
					data={suppliers}
					columns={columns}
				/>
			)}
		</PageShell>
	)
}
