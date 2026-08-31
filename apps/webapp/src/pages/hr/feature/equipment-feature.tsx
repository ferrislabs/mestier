import { useForm } from '@tanstack/react-form'
import type { ColumnDef } from '@tanstack/react-table'
import { Loader2, Package, Plus, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import {
	CreateButton,
	centsToEuros,
	MoneyCell,
	ReferenceTable,
	RowActions,
	RowIdentity,
	TextField,
} from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
} from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Equipment } from '#/hooks/use-reference-catalog'
import {
	useCreateEquipment,
	useDeleteEquipment,
	useReferenceCatalog,
	useUpdateEquipment,
} from '#/hooks/use-reference-catalog'
import type { EquipmentFormValues } from '#/pages/planning/types'

/**
 * Reference data for equipment and its hourly cost.
 *
 * It used to live in the organization settings; it is working data, not a
 * setting — its hourly cost feeds task profitability, so it belongs to the
 * module that schedules them.
 */
export function EquipmentFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<EquipmentSectionContent
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
		/>
	)
}

interface EquipmentSectionContentProps {
	organizationId: string
}

type Draft = {
	id: string
	values: EquipmentFormValues
} | null

function EquipmentSectionContent({
	organizationId,
}: EquipmentSectionContentProps) {
	const catalog = useReferenceCatalog(organizationId, {
		members: false,
		employeeProfiles: false,
		serviceRates: false,
		products: false,
	})
	const createEquipment = useCreateEquipment(organizationId)
	const updateEquipment = useUpdateEquipment()
	const deleteEquipment = useDeleteEquipment()

	const equipmentForm = useForm({
		defaultValues: { name: '', hourlyRate: '' } satisfies EquipmentFormValues,
		onSubmit: async ({ value }) => {
			await createEquipment.mutateAsync({
				path: { organization_id: organizationId },
				body: {
					name: value.name.trim(),
					hourly_rate_cents: eurosToCents(value.hourlyRate),
				},
			})
			equipmentForm.reset()
			setCreateDialogOpen(false)
		},
	})

	const handleCreateDialogOpenChange = (open: boolean) => {
		setCreateDialogOpen(open)
		if (!open) equipmentForm.reset()
	}

	const [search, setSearch] = useState('')
	const [createDialogOpen, setCreateDialogOpen] = useState(false)
	const [draft, setDraft] = useState<Draft>(null)
	const [isSaving, setIsSaving] = useState(false)

	const equipment = catalog.equipment.data?.data ?? []
	const isLoading = catalog.equipment.isLoading

	const error =
		catalog.equipment.error ??
		createEquipment.error ??
		updateEquipment.error ??
		deleteEquipment.error

	const normalizedSearch = search.trim().toLowerCase()
	const filteredEquipment = useMemo(
		() =>
			equipment.filter((item) =>
				item.name.toLowerCase().includes(normalizedSearch),
			),
		[equipment, normalizedSearch],
	)

	const handleSaveDraft = async () => {
		if (!draft) return
		setIsSaving(true)
		try {
			const item = equipment.find((entry) => entry.id === draft.id)
			if (item) {
				await updateEquipment.mutateAsync({
					path: { equipment_id: item.id },
					body: {
						name: draft.values.name.trim(),
						hourly_rate_cents: eurosToCents(draft.values.hourlyRate),
					},
				})
			}
			setDraft(null)
		} finally {
			setIsSaving(false)
		}
	}

	return (
		<equipmentForm.Subscribe selector={(state) => state.values}>
			{(equipmentValues) => (
				<PageShell>
					<PageHeader
						title="Matériel"
						description="Les ressources facturables et leur coût horaire, utilisés pour calculer la rentabilité d'une tâche."
						actions={
							<Button onClick={() => setCreateDialogOpen(true)}>
								<Plus />
								Ajouter une entrée
							</Button>
						}
					/>

					<MetricCard
						label="Matériel"
						value={equipment.length}
						hint="Ressources facturables"
						icon={<Package className="size-4" />}
					/>

					{error ? (
						<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
							{error.message}
						</div>
					) : null}

					<div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
						<div className="relative w-full lg:w-80">
							<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
							<Input
								type="search"
								value={search}
								onChange={(event) => setSearch(event.target.value)}
								placeholder="Rechercher dans le référentiel…"
								className="pl-9"
							/>
						</div>
					</div>

					{isLoading ? (
						<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
							<Loader2 className="size-5 animate-spin" />
							Chargement du matériel…
						</SectionCard>
					) : (
						<EquipmentTable
							data={filteredEquipment}
							draft={draft}
							isSaving={isSaving}
							onEdit={(item) =>
								setDraft({
									id: item.id,
									values: {
										name: item.name,
										hourlyRate: centsToEuros(item.hourly_rate_cents),
									},
								})
							}
							onDraftChange={(values) =>
								setDraft((current) =>
									current ? { ...current, values } : current,
								)
							}
							onCancel={() => setDraft(null)}
							onSave={handleSaveDraft}
							onDelete={(item) =>
								deleteEquipment.mutateAsync({
									path: { equipment_id: item.id },
								})
							}
						/>
					)}

					<Dialog
						open={createDialogOpen}
						onOpenChange={handleCreateDialogOpenChange}
					>
						<DialogContent className="flex max-h-[85vh] flex-col gap-0 overflow-hidden">
							<DialogHeader className="border-b pb-4">
								<DialogTitle>Ajouter une entrée</DialogTitle>
								<DialogDescription>
									Les montants sont saisis en euros et stockés en centimes côté
									API.
								</DialogDescription>
							</DialogHeader>
							<div className="flex-1 overflow-y-auto py-4">
								<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
									<TextField
										label="Nom"
										value={equipmentValues.name}
										onChange={(name) =>
											equipmentForm.setFieldValue('name', name)
										}
									/>
									<TextField
										label="Coût horaire"
										value={equipmentValues.hourlyRate}
										onChange={(hourlyRate) =>
											equipmentForm.setFieldValue('hourlyRate', hourlyRate)
										}
										inputMode="decimal"
										suffix="€/h"
									/>
								</div>
							</div>
							<DialogFooter className="border-t pt-4">
								<Button
									type="button"
									variant="ghost"
									onClick={() => handleCreateDialogOpenChange(false)}
								>
									Annuler
								</Button>
								<CreateButton
									isPending={createEquipment.isPending}
									onClick={() => void equipmentForm.handleSubmit()}
								/>
							</DialogFooter>
						</DialogContent>
					</Dialog>
				</PageShell>
			)}
		</equipmentForm.Subscribe>
	)
}

function eurosToCents(value: string): number {
	const normalized = value.replace(',', '.').trim()
	const parsed = Number.parseFloat(normalized)
	if (!Number.isFinite(parsed)) {
		return 0
	}
	return Math.round(parsed * 100)
}

interface EquipmentTableProps {
	data: Equipment[]
	draft: Draft
	isSaving: boolean
	onEdit: (equipment: Equipment) => void
	onDraftChange: (values: EquipmentFormValues) => void
	onCancel: () => void
	onSave: () => void
	onDelete: (equipment: Equipment) => Promise<unknown>
}

function EquipmentTable({
	data,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancel,
	onSave,
	onDelete,
}: EquipmentTableProps) {
	const columns = useMemo<ColumnDef<Equipment>[]>(
		() => [
			{
				header: 'Matériel',
				cell: ({ row }) =>
					draft && draft.id === row.original.id ? (
						<Input
							value={draft.values.name}
							onChange={(event) =>
								onDraftChange({ ...draft.values, name: event.target.value })
							}
						/>
					) : (
						<RowIdentity title={row.original.name} id={row.original.id} />
					),
			},
			{
				header: 'Coût',
				cell: ({ row }) =>
					draft && draft.id === row.original.id ? (
						<Input
							value={draft.values.hourlyRate}
							onChange={(event) =>
								onDraftChange({
									...draft.values,
									hourlyRate: event.target.value,
								})
							}
							inputMode="decimal"
						/>
					) : (
						<MoneyCell value={row.original.hourly_rate_cents} suffix="/h" />
					),
			},
			{
				id: 'actions',
				cell: ({ row }) => (
					<RowActions
						isEditing={!!draft && draft.id === row.original.id}
						isSaving={isSaving}
						onEdit={() => onEdit(row.original)}
						onCancel={onCancel}
						onSave={onSave}
						onDelete={() => onDelete(row.original)}
					/>
				),
			},
		],
		[draft, isSaving, onCancel, onDelete, onDraftChange, onEdit, onSave],
	)

	return (
		<ReferenceTable
			title="Matériel"
			description="Ressources matérielles facturables ou utilisées dans les opérations."
			emptyTitle="Aucun matériel trouvé"
			emptyDescription="Ajoutez une ressource pour la retrouver dans le référentiel de l'organisation."
			data={data}
			columns={columns}
		/>
	)
}
