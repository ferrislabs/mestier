import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import {
	BriefcaseBusiness,
	Building2,
	CircleDollarSign,
	Loader2,
	MoreHorizontal,
	Package,
	Plus,
	Save,
	Search,
	Trash2,
	Undo2,
	Users,
} from 'lucide-react'
import type * as React from 'react'
import { useMemo, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { Organization } from '#/hooks/use-organizations'
import type {
	Employee,
	Equipment,
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'
import type {
	EmployeeFormValues,
	EquipmentFormValues,
	OrganizationFormValues,
	ReferenceCatalogData,
	ReferenceTab,
	ServiceRateFormValues,
} from '#/pages/settings/types'

interface SettingsUIProps {
	organization: Organization
	isLoading: boolean
	error: string | null
	data: ReferenceCatalogData
	organizationForm: FormBinding<OrganizationFormValues>
	employeeForm: FormBinding<EmployeeFormValues>
	equipmentForm: FormBinding<EquipmentFormValues>
	serviceRateForm: FormBinding<ServiceRateFormValues>
	onUpdateEmployee: (
		employee: Employee,
		values: EmployeeFormValues,
	) => Promise<unknown>
	onDeleteEmployee: (employee: Employee) => Promise<unknown>
	onUpdateEquipment: (
		equipment: Equipment,
		values: EquipmentFormValues,
	) => Promise<unknown>
	onDeleteEquipment: (equipment: Equipment) => Promise<unknown>
	onUpdateServiceRate: (
		serviceRate: ServiceRate,
		values: ServiceRateFormValues,
	) => Promise<unknown>
	onDeleteServiceRate: (serviceRate: ServiceRate) => Promise<unknown>
}

interface FormBinding<T> {
	values: T
	isPending: boolean
	onChange: (patch: Partial<T>) => void
	onSubmit: () => void
}

type Draft =
	| { tab: 'employees'; id: string; values: EmployeeFormValues }
	| { tab: 'equipment'; id: string; values: EquipmentFormValues }
	| { tab: 'service-rates'; id: string; values: ServiceRateFormValues }
	| null

const TABS: {
	id: ReferenceTab
	label: string
	icon: typeof Users
}[] = [
	{ id: 'employees', label: 'Employés', icon: Users },
	{ id: 'equipment', label: 'Matériel', icon: Package },
	{ id: 'service-rates', label: 'Prestations', icon: BriefcaseBusiness },
]

const UNIT_LABELS: Record<ServiceRateUnit, string> = {
	HOUR: '€/h',
	ML: '€/ml',
	M2: '€/m²',
}

export function SettingsUI({
	organization,
	isLoading,
	error,
	data,
	organizationForm,
	employeeForm,
	equipmentForm,
	serviceRateForm,
	onUpdateEmployee,
	onDeleteEmployee,
	onUpdateEquipment,
	onDeleteEquipment,
	onUpdateServiceRate,
	onDeleteServiceRate,
}: SettingsUIProps) {
	const [activeTab, setActiveTab] = useState<ReferenceTab>('employees')
	const [search, setSearch] = useState('')
	const [draft, setDraft] = useState<Draft>(null)
	const [isSaving, setIsSaving] = useState(false)

	const normalizedSearch = search.trim().toLowerCase()
	const filteredData = useMemo(
		() => ({
			employees: data.employees.filter((employee) =>
				employee.name.toLowerCase().includes(normalizedSearch),
			),
			equipment: data.equipment.filter((item) =>
				item.name.toLowerCase().includes(normalizedSearch),
			),
			serviceRates: data.serviceRates.filter((serviceRate) =>
				serviceRate.label.toLowerCase().includes(normalizedSearch),
			),
		}),
		[data, normalizedSearch],
	)

	const totalHourlyCost =
		sum(data.employees.map((employee) => employee.hourly_rate_cents)) +
		sum(data.equipment.map((item) => item.hourly_rate_cents))

	const handleSaveDraft = async () => {
		if (!draft) return
		setIsSaving(true)
		try {
			if (draft.tab === 'employees') {
				const employee = data.employees.find((item) => item.id === draft.id)
				if (employee) await onUpdateEmployee(employee, draft.values)
			}
			if (draft.tab === 'equipment') {
				const equipment = data.equipment.find((item) => item.id === draft.id)
				if (equipment) await onUpdateEquipment(equipment, draft.values)
			}
			if (draft.tab === 'service-rates') {
				const serviceRate = data.serviceRates.find(
					(item) => item.id === draft.id,
				)
				if (serviceRate) await onUpdateServiceRate(serviceRate, draft.values)
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
				description="Configurez l’espace de travail, les équipes, les ressources et les tarifs utilisés par vos opérations."
			/>

			<OrganizationSection
				organization={organization}
				form={organizationForm}
			/>

			<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
				<MetricCard
					label="Employés"
					value={data.employees.length}
					hint="Taux horaires configurés"
					icon={<Users className="size-4" />}
				/>
				<MetricCard
					label="Matériel"
					value={data.equipment.length}
					hint="Ressources facturables"
					icon={<Package className="size-4" />}
				/>
				<MetricCard
					label="Coût horaire"
					value={formatMoney(totalHourlyCost)}
					hint="Équipes + ressources"
					icon={<CircleDollarSign className="size-4" />}
				/>
			</div>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
				<div className="flex flex-wrap gap-2">
					{TABS.map((tab) => {
						const active = tab.id === activeTab
						return (
							<button
								key={tab.id}
								type="button"
								onClick={() => {
									setActiveTab(tab.id)
									setDraft(null)
								}}
								className={`inline-flex h-9 items-center gap-2 rounded-lg border px-3 text-sm font-medium ${
									active
										? 'border-primary/30 bg-brand-soft text-primary'
										: 'border-border bg-card text-muted-foreground hover:bg-muted'
								}`}
							>
								<tab.icon className="size-4" />
								{tab.label}
							</button>
						)
					})}
				</div>

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
				activeTab={activeTab}
				employeeForm={employeeForm}
				equipmentForm={equipmentForm}
				serviceRateForm={serviceRateForm}
			/>

			{isLoading ? (
				<SettingsUI.Loading />
			) : activeTab === 'employees' ? (
				<EmployeeTable
					data={filteredData.employees}
					draft={draft}
					isSaving={isSaving}
					onEdit={(employee) =>
						setDraft({
							tab: 'employees',
							id: employee.id,
							values: {
								name: employee.name,
								hourlyRate: centsToEuros(employee.hourly_rate_cents),
								userId: employee.user_id ?? '',
							},
						})
					}
					onDraftChange={(values) =>
						setDraft((current) =>
							current?.tab === 'employees' ? { ...current, values } : current,
						)
					}
					onCancel={() => setDraft(null)}
					onSave={handleSaveDraft}
					onDelete={onDeleteEmployee}
				/>
			) : activeTab === 'equipment' ? (
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
			) : (
				<ServiceRateTable
					data={filteredData.serviceRates}
					draft={draft}
					isSaving={isSaving}
					onEdit={(serviceRate) =>
						setDraft({
							tab: 'service-rates',
							id: serviceRate.id,
							values: {
								label: serviceRate.label,
								unit: serviceRate.unit,
								rate: centsToEuros(serviceRate.rate_cents),
							},
						})
					}
					onDraftChange={(values) =>
						setDraft((current) =>
							current?.tab === 'service-rates'
								? { ...current, values }
								: current,
						)
					}
					onCancel={() => setDraft(null)}
					onSave={handleSaveDraft}
					onDelete={onDeleteServiceRate}
				/>
			)}
		</PageShell>
	)
}

interface OrganizationSectionProps {
	organization: Organization
	form: FormBinding<OrganizationFormValues>
}

function OrganizationSection({ organization, form }: OrganizationSectionProps) {
	const hasChanges =
		form.values.name.trim() !== organization.name ||
		form.values.slug.trim() !== organization.slug

	return (
		<SectionCard>
			<SectionHeader
				title="Organisation"
				description="Informations visibles dans l’application et utilisées pour identifier l’espace de travail."
				actions={
					<StatusBadge tone="brand">
						<Building2 className="mr-1 size-3" />
						{organization.slug}
					</StatusBadge>
				}
			/>
			<div className="grid grid-cols-1 gap-4 p-5 lg:grid-cols-[1fr_auto] lg:items-end">
				<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
					<TextField
						label="Nom"
						value={form.values.name}
						onChange={(name) => form.onChange({ name })}
						placeholder="Nom de l’entreprise"
					/>
					<TextField
						label="Identifiant"
						value={form.values.slug}
						onChange={(slug) => form.onChange({ slug: normalizeSlug(slug) })}
						placeholder="mon-entreprise"
						className="font-mono text-sm"
					/>
				</div>
				<Button
					type="button"
					onClick={form.onSubmit}
					disabled={form.isPending || !hasChanges}
					className="gap-2"
				>
					{form.isPending ? (
						<Loader2 className="size-4 animate-spin" />
					) : (
						<Save className="size-4" />
					)}
					Enregistrer
				</Button>
			</div>
		</SectionCard>
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
	activeTab: ReferenceTab
	employeeForm: FormBinding<EmployeeFormValues>
	equipmentForm: FormBinding<EquipmentFormValues>
	serviceRateForm: FormBinding<ServiceRateFormValues>
}

function CreateReferenceSection({
	activeTab,
	employeeForm,
	equipmentForm,
	serviceRateForm,
}: CreateReferenceSectionProps) {
	return (
		<SectionCard>
			<SectionHeader
				title="Ajouter une entrée"
				description="Les montants sont saisis en euros et stockés en centimes côté API."
			/>
			<div className="grid grid-cols-1 gap-4 p-5 lg:grid-cols-[1fr_auto] lg:items-end">
				{activeTab === 'employees' ? (
					<>
						<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
							<TextField
								label="Nom"
								value={employeeForm.values.name}
								onChange={(name) => employeeForm.onChange({ name })}
							/>
							<TextField
								label="Taux horaire"
								value={employeeForm.values.hourlyRate}
								onChange={(hourlyRate) => employeeForm.onChange({ hourlyRate })}
								inputMode="decimal"
								suffix="€/h"
							/>
							<TextField
								label="Compte Ferriskey"
								value={employeeForm.values.userId}
								onChange={(userId) => employeeForm.onChange({ userId })}
								placeholder="UUID optionnel"
							/>
						</div>
						<CreateButton
							isPending={employeeForm.isPending}
							onClick={employeeForm.onSubmit}
						/>
					</>
				) : activeTab === 'equipment' ? (
					<>
						<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
							<TextField
								label="Nom"
								value={equipmentForm.values.name}
								onChange={(name) => equipmentForm.onChange({ name })}
							/>
							<TextField
								label="Coût horaire"
								value={equipmentForm.values.hourlyRate}
								onChange={(hourlyRate) =>
									equipmentForm.onChange({ hourlyRate })
								}
								inputMode="decimal"
								suffix="€/h"
							/>
						</div>
						<CreateButton
							isPending={equipmentForm.isPending}
							onClick={equipmentForm.onSubmit}
						/>
					</>
				) : (
					<>
						<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
							<TextField
								label="Libellé"
								value={serviceRateForm.values.label}
								onChange={(label) => serviceRateForm.onChange({ label })}
							/>
							<div className="flex flex-col gap-2">
								<Label>Unité</Label>
								<Select
									value={serviceRateForm.values.unit}
									onValueChange={(unit) =>
										serviceRateForm.onChange({
											unit: unit as ServiceRateUnit,
										})
									}
								>
									<SelectTrigger className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										<SelectItem value="HOUR">Heure</SelectItem>
										<SelectItem value="ML">Mètre linéaire</SelectItem>
										<SelectItem value="M2">Mètre carré</SelectItem>
									</SelectContent>
								</Select>
							</div>
							<TextField
								label="Tarif"
								value={serviceRateForm.values.rate}
								onChange={(rate) => serviceRateForm.onChange({ rate })}
								inputMode="decimal"
								suffix={UNIT_LABELS[serviceRateForm.values.unit]}
							/>
						</div>
						<CreateButton
							isPending={serviceRateForm.isPending}
							onClick={serviceRateForm.onSubmit}
						/>
					</>
				)}
			</div>
		</SectionCard>
	)
}

function normalizeSlug(value: string): string {
	return value
		.toLowerCase()
		.trim()
		.replace(/\s+/g, '-')
		.replace(/[^a-z0-9-]/g, '')
		.replace(/-{2,}/g, '-')
}

interface EmployeeTableProps {
	data: Employee[]
	draft: Draft
	isSaving: boolean
	onEdit: (employee: Employee) => void
	onDraftChange: (values: EmployeeFormValues) => void
	onCancel: () => void
	onSave: () => void
	onDelete: (employee: Employee) => Promise<unknown>
}

function EmployeeTable({
	data,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancel,
	onSave,
	onDelete,
}: EmployeeTableProps) {
	const columns = useMemo<ColumnDef<Employee>[]>(
		() => [
			{
				header: 'Employé',
				cell: ({ row }) =>
					draft?.tab === 'employees' && draft.id === row.original.id ? (
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
				header: 'Compte',
				cell: ({ row }) =>
					draft?.tab === 'employees' && draft.id === row.original.id ? (
						<Input
							value={draft.values.userId}
							onChange={(event) =>
								onDraftChange({ ...draft.values, userId: event.target.value })
							}
							placeholder="UUID optionnel"
						/>
					) : row.original.user_id ? (
						<StatusBadge tone="success">lié</StatusBadge>
					) : (
						<StatusBadge>non lié</StatusBadge>
					),
			},
			{
				header: 'Taux',
				cell: ({ row }) =>
					draft?.tab === 'employees' && draft.id === row.original.id ? (
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
							draft?.tab === 'employees' && draft.id === row.original.id
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

	return <ReferenceTable title="Employés" data={data} columns={columns} />
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

	return <ReferenceTable title="Matériel" data={data} columns={columns} />
}

interface ServiceRateTableProps {
	data: ServiceRate[]
	draft: Draft
	isSaving: boolean
	onEdit: (serviceRate: ServiceRate) => void
	onDraftChange: (values: ServiceRateFormValues) => void
	onCancel: () => void
	onSave: () => void
	onDelete: (serviceRate: ServiceRate) => Promise<unknown>
}

function ServiceRateTable({
	data,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancel,
	onSave,
	onDelete,
}: ServiceRateTableProps) {
	const columns = useMemo<ColumnDef<ServiceRate>[]>(
		() => [
			{
				header: 'Prestation',
				cell: ({ row }) =>
					draft?.tab === 'service-rates' && draft.id === row.original.id ? (
						<Input
							value={draft.values.label}
							onChange={(event) =>
								onDraftChange({ ...draft.values, label: event.target.value })
							}
						/>
					) : (
						<RowIdentity title={row.original.label} id={row.original.id} />
					),
			},
			{
				header: 'Unité',
				cell: ({ row }) =>
					draft?.tab === 'service-rates' && draft.id === row.original.id ? (
						<Select
							value={draft.values.unit}
							onValueChange={(unit) =>
								onDraftChange({
									...draft.values,
									unit: unit as ServiceRateUnit,
								})
							}
						>
							<SelectTrigger className="w-36">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value="HOUR">Heure</SelectItem>
								<SelectItem value="ML">Mètre linéaire</SelectItem>
								<SelectItem value="M2">Mètre carré</SelectItem>
							</SelectContent>
						</Select>
					) : (
						<StatusBadge tone="brand">
							{UNIT_LABELS[row.original.unit]}
						</StatusBadge>
					),
			},
			{
				header: 'Tarif',
				cell: ({ row }) =>
					draft?.tab === 'service-rates' && draft.id === row.original.id ? (
						<Input
							value={draft.values.rate}
							onChange={(event) =>
								onDraftChange({ ...draft.values, rate: event.target.value })
							}
							inputMode="decimal"
						/>
					) : (
						<MoneyCell
							value={row.original.rate_cents}
							suffix={unitSuffix(row.original.unit)}
						/>
					),
			},
			{
				id: 'actions',
				cell: ({ row }) => (
					<RowActions
						isEditing={
							draft?.tab === 'service-rates' && draft.id === row.original.id
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

	return <ReferenceTable title="Prestations" data={data} columns={columns} />
}

interface ReferenceTableProps<T> {
	title: string
	data: T[]
	columns: ColumnDef<T>[]
}

function ReferenceTable<T>({ title, data, columns }: ReferenceTableProps<T>) {
	const table = useReactTable({
		data,
		columns,
		getCoreRowModel: getCoreRowModel(),
	})

	return (
		<SectionCard>
			<SectionHeader
				title={`${title} (${data.length})`}
				description="Modifiez une ligne ou supprimez-la du référentiel actif."
			/>
			<div className="overflow-x-auto">
				<table className="w-full min-w-[720px] border-collapse text-sm">
					<thead>
						{table.getHeaderGroups().map((headerGroup) => (
							<tr key={headerGroup.id} className="border-b bg-muted/50">
								{headerGroup.headers.map((header) => (
									<th
										key={header.id}
										className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground"
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
						{table.getRowModel().rows.length === 0 ? (
							<tr>
								<td
									colSpan={columns.length}
									className="px-5 py-12 text-center text-sm text-muted-foreground"
								>
									Aucune entrée trouvée.
								</td>
							</tr>
						) : (
							table.getRowModel().rows.map((row) => (
								<tr key={row.id} className="border-b last:border-b-0">
									{row.getVisibleCells().map((cell) => (
										<td key={cell.id} className="px-5 py-3 align-middle">
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
		</SectionCard>
	)
}

interface RowActionsProps {
	isEditing: boolean
	isSaving: boolean
	onEdit: () => void
	onCancel: () => void
	onSave: () => void
	onDelete: () => void
}

function RowActions({
	isEditing,
	isSaving,
	onEdit,
	onCancel,
	onSave,
	onDelete,
}: RowActionsProps) {
	if (isEditing) {
		return (
			<div className="flex justify-end gap-1">
				<Button size="icon-sm" variant="ghost" onClick={onCancel}>
					<Undo2 />
					<span className="sr-only">Annuler</span>
				</Button>
				<Button size="icon-sm" onClick={onSave} disabled={isSaving}>
					{isSaving ? <Loader2 className="animate-spin" /> : <Save />}
					<span className="sr-only">Enregistrer</span>
				</Button>
			</div>
		)
	}

	return (
		<div className="flex justify-end">
			<DropdownMenu>
				<DropdownMenuTrigger asChild>
					<Button size="icon-sm" variant="ghost">
						<MoreHorizontal />
						<span className="sr-only">Actions</span>
					</Button>
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end">
					<DropdownMenuItem onClick={onEdit}>Modifier</DropdownMenuItem>
					<DropdownMenuSeparator />
					<DropdownMenuItem variant="destructive" onClick={onDelete}>
						<Trash2 />
						Supprimer
					</DropdownMenuItem>
				</DropdownMenuContent>
			</DropdownMenu>
		</div>
	)
}

interface TextFieldProps extends React.InputHTMLAttributes<HTMLInputElement> {
	label: string
	value: string
	onChange: (value: string) => void
	suffix?: string
}

function TextField({
	label,
	value,
	onChange,
	suffix,
	...props
}: TextFieldProps) {
	const id = label.toLowerCase().replace(/\s+/g, '-')
	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			<div className="relative">
				<Input
					id={id}
					value={value}
					onChange={(event) => onChange(event.target.value)}
					className={suffix ? 'pr-14' : undefined}
					{...props}
				/>
				{suffix ? (
					<span className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs font-medium text-muted-foreground">
						{suffix}
					</span>
				) : null}
			</div>
		</div>
	)
}

function CreateButton({
	isPending,
	onClick,
}: {
	isPending: boolean
	onClick: () => void
}) {
	return (
		<Button onClick={onClick} disabled={isPending}>
			{isPending ? <Loader2 className="animate-spin" /> : <Plus />}
			Ajouter
		</Button>
	)
}

function RowIdentity({ title, id }: { title: string; id: string }) {
	return (
		<div className="min-w-0">
			<p className="truncate font-medium">{title}</p>
			<p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">
				id: {id}
			</p>
		</div>
	)
}

function MoneyCell({ value, suffix }: { value: number; suffix: string }) {
	return (
		<span className="font-medium tabular-nums">
			{formatMoney(value)}
			<span className="text-muted-foreground">{suffix}</span>
		</span>
	)
}

function centsToEuros(value: number): string {
	return (value / 100).toFixed(2).replace('.', ',')
}

function formatMoney(value: number): string {
	return new Intl.NumberFormat('fr-FR', {
		style: 'currency',
		currency: 'EUR',
	}).format(value / 100)
}

function unitSuffix(unit: ServiceRateUnit): string {
	if (unit === 'HOUR') return '/h'
	if (unit === 'ML') return '/ml'
	return '/m²'
}

function sum(values: number[]): number {
	return values.reduce((total, value) => total + value, 0)
}
