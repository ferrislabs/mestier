import { useForm } from '@tanstack/react-form'
import type { ColumnDef } from '@tanstack/react-table'
import { AlertCircle, Loader2, Package, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Input } from '#/components/ui/input'
import { MetricCard, SectionCard, SectionHeader } from '#/components/ui/surface'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import type { Equipment } from '#/hooks/use-reference-catalog'
import {
	useCreateEquipment,
	useDeleteEquipment,
	useReferenceCatalog,
	useUpdateEquipment,
} from '#/hooks/use-reference-catalog'
import type { EquipmentFormValues } from '#/pages/settings/types'
import {
	CreateButton,
	centsToEuros,
	type FormBinding,
	MoneyCell,
	ReferenceTable,
	RowActions,
	RowIdentity,
	TextField,
} from '#/pages/settings/ui/primitives'

export function EquipmentSection() {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le référentiel nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

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
		employees: false,
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
		},
	})

	const [search, setSearch] = useState('')
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
				<div className="flex flex-col gap-6">
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

					<CreateReferenceSection
						equipmentForm={{
							values: equipmentValues,
							isPending: createEquipment.isPending,
							onChange: (patch) => {
								for (const key of Object.keys(
									patch,
								) as (keyof EquipmentFormValues)[]) {
									equipmentForm.setFieldValue(key, patch[key] ?? '')
								}
							},
							onSubmit: () => void equipmentForm.handleSubmit(),
						}}
					/>

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
				</div>
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

interface CreateReferenceSectionProps {
	equipmentForm: FormBinding<EquipmentFormValues>
}

function CreateReferenceSection({
	equipmentForm,
}: CreateReferenceSectionProps) {
	return (
		<SectionCard>
			<SectionHeader
				title="Ajouter une entrée"
				description="Les montants sont saisis en euros et stockés en centimes côté API."
			/>
			<div className="grid grid-cols-1 gap-4 p-5 lg:grid-cols-[1fr_auto] lg:items-end">
				<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
					<TextField
						label="Nom"
						value={equipmentForm.values.name}
						onChange={(name) => equipmentForm.onChange({ name })}
					/>
					<TextField
						label="Coût horaire"
						value={equipmentForm.values.hourlyRate}
						onChange={(hourlyRate) => equipmentForm.onChange({ hourlyRate })}
						inputMode="decimal"
						suffix="€/h"
					/>
				</div>
				<CreateButton
					isPending={equipmentForm.isPending}
					onClick={equipmentForm.onSubmit}
				/>
			</div>
		</SectionCard>
	)
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
