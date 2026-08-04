import type { ColumnDef } from '@tanstack/react-table'
import { Loader2, Package, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Input } from '#/components/ui/input'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
} from '#/components/ui/surface'
import type { Organization } from '#/hooks/use-organizations'
import type { Equipment } from '#/hooks/use-reference-catalog'
import type {
	EquipmentFormValues,
	ReferenceCatalogData,
} from '#/pages/settings/types'
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

interface SettingsUIProps {
	organization: Organization
	isLoading: boolean
	error: string | null
	data: ReferenceCatalogData
	equipmentForm: FormBinding<EquipmentFormValues>
	onUpdateEquipment: (
		equipment: Equipment,
		values: EquipmentFormValues,
	) => Promise<unknown>
	onDeleteEquipment: (equipment: Equipment) => Promise<unknown>
}

type Draft = {
	tab: 'equipment'
	id: string
	values: EquipmentFormValues
} | null

export function SettingsUI({
	organization,
	isLoading,
	error,
	data,
	equipmentForm,
	onUpdateEquipment,
	onDeleteEquipment,
}: SettingsUIProps) {
	const [search, setSearch] = useState('')
	const [draft, setDraft] = useState<Draft>(null)
	const [isSaving, setIsSaving] = useState(false)

	const normalizedSearch = search.trim().toLowerCase()
	const filteredData = useMemo(
		() => ({
			equipment: data.equipment.filter((item) =>
				item.name.toLowerCase().includes(normalizedSearch),
			),
		}),
		[data, normalizedSearch],
	)

	const handleSaveDraft = async () => {
		if (!draft) return
		setIsSaving(true)
		try {
			if (draft.tab === 'equipment') {
				const equipment = data.equipment.find((item) => item.id === draft.id)
				if (equipment) await onUpdateEquipment(equipment, draft.values)
			}
			setDraft(null)
		} finally {
			setIsSaving(false)
		}
	}

	return (
		<PageShell>
			<PageHeader
				eyebrow={organization.name}
				title="Paramètres"
				description="Configurez l’espace de travail et les ressources internes de l’organisation."
			/>

			<MetricCard
				label="Matériel"
				value={data.equipment.length}
				hint="Ressources facturables"
				icon={<Package className="size-4" />}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
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

			<CreateReferenceSection equipmentForm={equipmentForm} />

			{isLoading ? (
				<SettingsUI.Loading />
			) : (
				<EquipmentTable
					data={filteredData.equipment}
					draft={draft}
					isSaving={isSaving}
					onEdit={(equipment) =>
						setDraft({
							tab: 'equipment',
							id: equipment.id,
							values: {
								name: equipment.name,
								hourlyRate: centsToEuros(equipment.hourly_rate_cents),
							},
						})
					}
					onDraftChange={(values) =>
						setDraft((current) =>
							current?.tab === 'equipment' ? { ...current, values } : current,
						)
					}
					onCancel={() => setDraft(null)}
					onSave={handleSaveDraft}
					onDelete={onDeleteEquipment}
				/>
			)}
		</PageShell>
	)
}

SettingsUI.Loading = function SettingsLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement des paramètres…
			</SectionCard>
		</PageShell>
	)
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
					draft?.tab === 'equipment' && draft.id === row.original.id ? (
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
					draft?.tab === 'equipment' && draft.id === row.original.id ? (
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
						isEditing={
							draft?.tab === 'equipment' && draft.id === row.original.id
						}
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
