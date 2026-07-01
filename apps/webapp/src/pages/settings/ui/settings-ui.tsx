import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import {
	Building2,
	ChevronDown,
	FileText,
	Landmark,
	Loader2,
	MoreHorizontal,
	Package,
	Plus,
	Receipt,
	Save,
	Search,
	Trash2,
	Undo2,
	Users,
} from 'lucide-react'
import type * as React from 'react'
import { useMemo, useState } from 'react'
import {
	CustomFieldsEditor,
	recordToCustomFields,
} from '#/components/custom-fields-editor'
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
import { Textarea } from '#/components/ui/textarea'
import type { OrganizationContext } from '#/hooks/use-organization-contexts'
import type { Organization } from '#/hooks/use-organizations'
import type {
	Employee,
	Equipment,
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'
import { LEGAL_MENTION_PRESETS } from '#/lib/legal-mention-presets'
import type {
	BillingFormValues,
	EmitterContextFormValues,
	EmployeeFormValues,
	EquipmentFormValues,
	LegalMentionFormValues,
	LegalMentionTemplate,
	OrganizationFormValues,
	Product,
	ProductCatalogFormValues,
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
	billingForm: FormBinding<BillingFormValues>
	employeeForm: FormBinding<EmployeeFormValues>
	equipmentForm: FormBinding<EquipmentFormValues>
	serviceRateForm: FormBinding<ServiceRateFormValues>
	productForm: FormBinding<ProductCatalogFormValues>
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
	onUpdateProduct: (
		product: Product,
		values: ProductCatalogFormValues,
	) => Promise<unknown>
	onDeleteProduct: (product: Product) => Promise<unknown>
	onCreateLegalMentionTemplate: (
		values: LegalMentionFormValues,
	) => Promise<unknown>
	onUpdateLegalMentionTemplate: (
		template: LegalMentionTemplate,
		values: LegalMentionFormValues,
	) => Promise<unknown>
	onDeleteLegalMentionTemplate: (
		template: LegalMentionTemplate,
	) => Promise<unknown>
	onCreateOrganizationContext: (
		values: EmitterContextFormValues,
	) => Promise<unknown>
	onUpdateOrganizationContext: (
		context: OrganizationContext,
		values: EmitterContextFormValues,
	) => Promise<unknown>
	onDeleteOrganizationContext: (
		context: OrganizationContext,
	) => Promise<unknown>
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
	| { tab: 'products'; id: string; values: ProductCatalogFormValues }
	| null

const TABS: {
	id: ReferenceTab
	label: string
	icon: typeof Users
}[] = [
	{ id: 'employees', label: 'Employés', icon: Users },
	{ id: 'equipment', label: 'Matériel', icon: Package },
]

const UNIT_LABELS: Record<ServiceRateUnit, string> = {
	HOUR: '€/h',
	ML: '€/ml',
	M2: '€/m²',
}

const PRODUCT_UNIT_LABELS: Record<ServiceRateUnit, string> = {
	HOUR: '€/unité',
	ML: '€/ml',
	M2: '€/m²',
}

export function SettingsUI({
	organization,
	isLoading,
	error,
	data,
	organizationForm,
	billingForm,
	employeeForm,
	equipmentForm,
	serviceRateForm,
	productForm,
	onUpdateEmployee,
	onDeleteEmployee,
	onUpdateEquipment,
	onDeleteEquipment,
	onUpdateServiceRate,
	onDeleteServiceRate,
	onUpdateProduct,
	onDeleteProduct,
	onCreateLegalMentionTemplate,
	onUpdateLegalMentionTemplate,
	onDeleteLegalMentionTemplate,
	onCreateOrganizationContext,
	onUpdateOrganizationContext,
	onDeleteOrganizationContext,
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
			products: data.products.filter((product) => {
				return (
					product.name.toLowerCase().includes(normalizedSearch) ||
					(product.sku ?? '').toLowerCase().includes(normalizedSearch) ||
					(product.description ?? '').toLowerCase().includes(normalizedSearch)
				)
			}),
		}),
		[data, normalizedSearch],
	)

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
			if (draft.tab === 'products') {
				const product = data.products.find((item) => item.id === draft.id)
				if (product) await onUpdateProduct(product, draft.values)
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
				description="Configurez l’espace de travail, les équipes et les ressources internes de l’organisation."
			/>

			<OrganizationSection
				organization={organization}
				form={organizationForm}
			/>

			<EmitterContextSection
				contexts={data.organizationContexts}
				onCreate={onCreateOrganizationContext}
				onUpdate={onUpdateOrganizationContext}
				onDelete={onDeleteOrganizationContext}
			/>

			<BillingSection form={billingForm} />

			<LegalMentionSection
				templates={data.legalMentionTemplates}
				onCreate={onCreateLegalMentionTemplate}
				onUpdate={onUpdateLegalMentionTemplate}
				onDelete={onDeleteLegalMentionTemplate}
			/>

			<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
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
				productForm={productForm}
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
			) : activeTab === 'service-rates' ? (
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
								vatRate: serviceRate.vat_rate,
								customFields: recordToCustomFields(serviceRate.custom_fields),
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
			) : (
				<ProductTable
					data={filteredData.products}
					draft={draft}
					isSaving={isSaving}
					onEdit={(product) =>
						setDraft({
							tab: 'products',
							id: product.id,
							values: {
								name: product.name,
								sku: product.sku ?? '',
								unit: product.unit,
								unitPrice: centsToEuros(product.unit_price_cents),
								vatRate: product.vat_rate,
								description: product.description ?? '',
								customFields: recordToCustomFields(product.custom_fields),
							},
						})
					}
					onDraftChange={(values) =>
						setDraft((current) =>
							current?.tab === 'products' ? { ...current, values } : current,
						)
					}
					onCancel={() => setDraft(null)}
					onSave={handleSaveDraft}
					onDelete={onDeleteProduct}
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
	productForm: FormBinding<ProductCatalogFormValues>
}

function CreateReferenceSection({
	activeTab,
	employeeForm,
	equipmentForm,
	serviceRateForm,
	productForm,
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
				) : activeTab === 'service-rates' ? (
					<>
						<div className="flex flex-col gap-4 flex-1">
							<div className="grid grid-cols-1 gap-4 md:grid-cols-4">
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
								<TextField
									label="TVA"
									value={serviceRateForm.values.vatRate}
									onChange={(vatRate) => serviceRateForm.onChange({ vatRate })}
									inputMode="decimal"
									suffix="%"
								/>
							</div>
							<CustomFieldsEditor
								fields={serviceRateForm.values.customFields}
								onChange={(customFields) =>
									serviceRateForm.onChange({ customFields })
								}
							/>
						</div>
						<CreateButton
							isPending={serviceRateForm.isPending}
							onClick={serviceRateForm.onSubmit}
						/>
					</>
				) : (
					<>
						<div className="flex flex-col gap-4 flex-1">
							<div className="grid grid-cols-1 gap-4 md:grid-cols-6">
								<TextField
									label="Produit"
									value={productForm.values.name}
									onChange={(name) => productForm.onChange({ name })}
								/>
								<TextField
									label="Référence"
									value={productForm.values.sku}
									onChange={(sku) => productForm.onChange({ sku })}
									placeholder="Optionnel"
								/>
								<div className="flex flex-col gap-2">
									<Label>Unité</Label>
									<Select
										value={productForm.values.unit}
										onValueChange={(unit) =>
											productForm.onChange({ unit: unit as ServiceRateUnit })
										}
									>
										<SelectTrigger className="w-full">
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="ML">Mètre linéaire</SelectItem>
											<SelectItem value="M2">Mètre carré</SelectItem>
											<SelectItem value="HOUR">Unité</SelectItem>
										</SelectContent>
									</Select>
								</div>
								<TextField
									label="Prix"
									value={productForm.values.unitPrice}
									onChange={(unitPrice) => productForm.onChange({ unitPrice })}
									inputMode="decimal"
									suffix={PRODUCT_UNIT_LABELS[productForm.values.unit]}
								/>
								<TextField
									label="TVA"
									value={productForm.values.vatRate}
									onChange={(vatRate) => productForm.onChange({ vatRate })}
									inputMode="decimal"
									suffix="%"
								/>
								<TextField
									label="Description"
									value={productForm.values.description}
									onChange={(description) =>
										productForm.onChange({ description })
									}
									placeholder="Optionnel"
								/>
							</div>
							<CustomFieldsEditor
								fields={productForm.values.customFields}
								onChange={(customFields) =>
									productForm.onChange({ customFields })
								}
							/>
						</div>
						<CreateButton
							isPending={productForm.isPending}
							onClick={productForm.onSubmit}
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

	return (
		<ReferenceTable
			title="Employés"
			description="Taux horaires et rattachements aux comptes utilisateurs."
			emptyTitle="Aucun employé trouvé"
			emptyDescription="Ajoutez un employé pour le rendre disponible dans les prochains workflows opérationnels."
			data={data}
			columns={columns}
		/>
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
							{PRODUCT_UNIT_LABELS[row.original.unit]}
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
				header: 'TVA',
				cell: ({ row }) =>
					draft?.tab === 'service-rates' && draft.id === row.original.id ? (
						<div className="relative w-24">
							<Input
								value={draft.values.vatRate}
								onChange={(event) =>
									onDraftChange({
										...draft.values,
										vatRate: event.target.value,
									})
								}
								inputMode="decimal"
								className="pr-6"
							/>
							<span className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-xs text-muted-foreground">
								%
							</span>
						</div>
					) : (
						<span className="text-sm">{row.original.vat_rate}%</span>
					),
			},
			{
				header: 'Champs perso',
				cell: ({ row }) =>
					draft?.tab === 'service-rates' && draft.id === row.original.id ? (
						<CustomFieldsEditor
							fields={draft.values.customFields}
							onChange={(customFields) =>
								onDraftChange({ ...draft.values, customFields })
							}
						/>
					) : (
						<span className="text-sm text-muted-foreground">
							{Object.keys(row.original.custom_fields).length > 0
								? `${Object.keys(row.original.custom_fields).length} champ(s)`
								: '—'}
						</span>
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

	return (
		<ReferenceTable
			title="Services"
			description="Prestations réutilisables dans les devis et futures pièces commerciales."
			emptyTitle="Aucun service trouvé"
			emptyDescription="Ajoutez une prestation pour accélérer la saisie des devis."
			data={data}
			columns={columns}
		/>
	)
}

interface ProductTableProps {
	data: Product[]
	draft: Draft
	isSaving: boolean
	onEdit: (product: Product) => void
	onDraftChange: (values: ProductCatalogFormValues) => void
	onCancel: () => void
	onSave: () => void
	onDelete: (product: Product) => Promise<unknown>
}

function ProductTable({
	data,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancel,
	onSave,
	onDelete,
}: ProductTableProps) {
	const columns = useMemo<ColumnDef<Product>[]>(
		() => [
			{
				header: 'Produit',
				cell: ({ row }) =>
					draft?.tab === 'products' && draft.id === row.original.id ? (
						<Input
							value={draft.values.name}
							onChange={(event) =>
								onDraftChange({ ...draft.values, name: event.target.value })
							}
						/>
					) : (
						<ProductIdentity product={row.original} />
					),
			},
			{
				header: 'Référence',
				cell: ({ row }) =>
					draft?.tab === 'products' && draft.id === row.original.id ? (
						<Input
							value={draft.values.sku}
							onChange={(event) =>
								onDraftChange({ ...draft.values, sku: event.target.value })
							}
							placeholder="Optionnel"
						/>
					) : row.original.sku ? (
						<StatusBadge tone="brand">{row.original.sku}</StatusBadge>
					) : (
						<StatusBadge>sans réf.</StatusBadge>
					),
			},
			{
				header: 'Unité',
				cell: ({ row }) =>
					draft?.tab === 'products' && draft.id === row.original.id ? (
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
								<SelectItem value="ML">Mètre linéaire</SelectItem>
								<SelectItem value="M2">Mètre carré</SelectItem>
								<SelectItem value="HOUR">Unité</SelectItem>
							</SelectContent>
						</Select>
					) : (
						<StatusBadge tone="brand">
							{UNIT_LABELS[row.original.unit]}
						</StatusBadge>
					),
			},
			{
				header: 'Prix',
				cell: ({ row }) =>
					draft?.tab === 'products' && draft.id === row.original.id ? (
						<Input
							value={draft.values.unitPrice}
							onChange={(event) =>
								onDraftChange({
									...draft.values,
									unitPrice: event.target.value,
								})
							}
							inputMode="decimal"
						/>
					) : (
						<MoneyCell
							value={row.original.unit_price_cents}
							suffix={productUnitSuffix(row.original.unit)}
						/>
					),
			},
			{
				header: 'TVA',
				cell: ({ row }) =>
					draft?.tab === 'products' && draft.id === row.original.id ? (
						<div className="relative w-24">
							<Input
								value={draft.values.vatRate}
								onChange={(event) =>
									onDraftChange({
										...draft.values,
										vatRate: event.target.value,
									})
								}
								inputMode="decimal"
								className="pr-6"
							/>
							<span className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-xs text-muted-foreground">
								%
							</span>
						</div>
					) : (
						<span className="text-sm">{row.original.vat_rate}%</span>
					),
			},
			{
				header: 'Description',
				cell: ({ row }) =>
					draft?.tab === 'products' && draft.id === row.original.id ? (
						<Input
							value={draft.values.description}
							onChange={(event) =>
								onDraftChange({
									...draft.values,
									description: event.target.value,
								})
							}
							placeholder="Optionnel"
						/>
					) : (
						<p className="max-w-64 truncate text-sm text-muted-foreground">
							{row.original.description || 'Aucune description'}
						</p>
					),
			},
			{
				header: 'Champs perso',
				cell: ({ row }) =>
					draft?.tab === 'products' && draft.id === row.original.id ? (
						<CustomFieldsEditor
							fields={draft.values.customFields}
							onChange={(customFields) =>
								onDraftChange({ ...draft.values, customFields })
							}
						/>
					) : (
						<span className="text-sm text-muted-foreground">
							{Object.keys(row.original.custom_fields).length > 0
								? `${Object.keys(row.original.custom_fields).length} champ(s)`
								: '—'}
						</span>
					),
			},
			{
				id: 'actions',
				cell: ({ row }) => (
					<RowActions
						isEditing={
							draft?.tab === 'products' && draft.id === row.original.id
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
			title="Produits"
			description="Articles vendables avec référence, unité, prix et description préremplie."
			emptyTitle="Aucun produit trouvé"
			emptyDescription="Ajoutez un produit pour l'insérer rapidement dans les devis."
			data={data}
			columns={columns}
		/>
	)
}

interface ReferenceTableProps<T> {
	title: string
	description: string
	emptyTitle: string
	emptyDescription: string
	data: T[]
	columns: ColumnDef<T>[]
}

function ReferenceTable<T>({
	title,
	description,
	emptyTitle,
	emptyDescription,
	data,
	columns,
}: ReferenceTableProps<T>) {
	const table = useReactTable({
		data,
		columns,
		getCoreRowModel: getCoreRowModel(),
	})

	return (
		<SectionCard>
			<SectionHeader
				title={`${title} (${data.length})`}
				description={description}
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
								<td colSpan={columns.length} className="px-5 py-12 text-center">
									<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
										<p className="font-medium">{emptyTitle}</p>
										<p className="text-sm text-muted-foreground">
											{emptyDescription}
										</p>
									</div>
								</td>
							</tr>
						) : (
							table.getRowModel().rows.map((row) => (
								<tr
									key={row.id}
									className="group border-b transition hover:bg-muted/35 hover:shadow-xs last:border-b-0"
								>
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
		<div className="flex justify-end opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
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

interface TextFieldProps
	extends Omit<
		React.InputHTMLAttributes<HTMLInputElement>,
		'value' | 'onChange'
	> {
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

function ProductIdentity({ product }: { product: Product }) {
	return (
		<div className="min-w-0">
			<div className="flex min-w-0 items-center gap-2">
				<p className="truncate font-medium">{product.name}</p>
				{product.sku ? (
					<StatusBadge tone="brand">{product.sku}</StatusBadge>
				) : null}
			</div>
			<p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">
				id: {product.id}
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

function productUnitSuffix(unit: ServiceRateUnit): string {
	if (unit === 'HOUR') return '/unité'
	if (unit === 'ML') return '/ml'
	return '/m²'
}

// ─── Billing Section ─────────────────────────────────────────────────────────

interface BillingSectionProps {
	form: FormBinding<BillingFormValues>
}

function BillingSection({ form }: BillingSectionProps) {
	return (
		<SectionCard>
			<SectionHeader
				title="Facturation"
				description="Paramètres appliqués aux devis, factures et exports PDF."
				actions={
					<StatusBadge tone="brand">
						<Receipt className="mr-1 size-3" />
						Facturation
					</StatusBadge>
				}
			/>
			<div className="flex flex-col gap-6 p-5">
				<fieldset className="flex flex-col gap-3">
					<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
						Paiement
					</legend>
					<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
						<TextField
							label="Délai de paiement"
							value={form.values.paymentTermsDays}
							onChange={(v) => form.onChange({ paymentTermsDays: v })}
							inputMode="numeric"
							suffix="jours"
						/>
						<TextField
							label="Taux pénalités retard"
							value={form.values.latePenaltyRate}
							onChange={(v) => form.onChange({ latePenaltyRate: v })}
							inputMode="decimal"
							suffix="%"
						/>
						<TextField
							label="Indemnité forfaitaire recouvrement"
							value={form.values.recoveryIndemnityEuros}
							onChange={(v) => form.onChange({ recoveryIndemnityEuros: v })}
							inputMode="decimal"
							suffix="€"
						/>
					</div>
				</fieldset>

				<fieldset className="flex flex-col gap-3">
					<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
						Acompte par défaut
					</legend>
					<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
						<div className="flex flex-col gap-2">
							<Label>Type d&apos;acompte</Label>
							<Select
								value={form.values.defaultDepositBasis ?? ''}
								onValueChange={(v) =>
									form.onChange({ defaultDepositBasis: v || null })
								}
							>
								<SelectTrigger className="w-full">
									<SelectValue placeholder="— Aucun —" />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="PERCENT">Pourcentage</SelectItem>
									<SelectItem value="FIXED">Montant fixe (€)</SelectItem>
								</SelectContent>
							</Select>
						</div>
						<TextField
							label="Valeur de l'acompte"
							value={form.values.defaultDepositValue ?? ''}
							onChange={(v) =>
								form.onChange({ defaultDepositValue: v || null })
							}
							inputMode="decimal"
							suffix={form.values.defaultDepositBasis === 'PERCENT' ? '%' : '€'}
							disabled={!form.values.defaultDepositBasis}
						/>
					</div>
				</fieldset>

				<fieldset className="flex flex-col gap-3">
					<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
						TVA
					</legend>
					<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
						<TextField
							label="Taux de TVA par défaut"
							value={form.values.defaultVatRate}
							onChange={(v) => form.onChange({ defaultVatRate: v })}
							inputMode="decimal"
							suffix="%"
						/>
					</div>
				</fieldset>

				<fieldset className="flex flex-col gap-3">
					<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
						Identifiants légaux
					</legend>
					<div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
						<TextField
							label="SIRET"
							value={form.values.siret}
							onChange={(v) => form.onChange({ siret: v })}
							placeholder="14 chiffres"
						/>
						<TextField
							label="RCS"
							value={form.values.rcs}
							onChange={(v) => form.onChange({ rcs: v })}
							placeholder="Ville + n° d'immatriculation"
						/>
						<TextField
							label="APE / NAF"
							value={form.values.ape}
							onChange={(v) => form.onChange({ ape: v })}
							placeholder="ex. 4321A"
						/>
						<TextField
							label="N° TVA intracommunautaire"
							value={form.values.vatIntracom}
							onChange={(v) => form.onChange({ vatIntracom: v })}
							placeholder="ex. FR12345678901"
						/>
					</div>
				</fieldset>

				<fieldset className="flex flex-col gap-3">
					<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
						Banque
					</legend>
					<div className="grid grid-cols-1 gap-4 md:grid-cols-2">
						<TextField
							label="IBAN"
							value={form.values.iban}
							onChange={(v) => form.onChange({ iban: v })}
							placeholder="FR76 …"
						/>
						<TextField
							label="BIC"
							value={form.values.bic}
							onChange={(v) => form.onChange({ bic: v })}
							placeholder="ex. BNPAFRPP"
						/>
					</div>
				</fieldset>

				<fieldset className="flex flex-col gap-3">
					<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
						Pied de page
					</legend>
					<div className="flex flex-col gap-2">
						<Label htmlFor="billing-footer">Texte de pied de page</Label>
						<Textarea
							id="billing-footer"
							value={form.values.footer}
							onChange={(e) => form.onChange({ footer: e.target.value })}
							rows={3}
							placeholder="Texte libre affiché en bas des documents…"
						/>
					</div>
				</fieldset>

				<div className="flex justify-end">
					<Button
						type="button"
						onClick={form.onSubmit}
						disabled={form.isPending}
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
			</div>
		</SectionCard>
	)
}

// ─── Legal Mention Section ────────────────────────────────────────────────────

interface LegalMentionSectionProps {
	templates: LegalMentionTemplate[]
	onCreate: (values: LegalMentionFormValues) => Promise<unknown>
	onUpdate: (
		template: LegalMentionTemplate,
		values: LegalMentionFormValues,
	) => Promise<unknown>
	onDelete: (template: LegalMentionTemplate) => Promise<unknown>
}

type LegalMentionDraft = {
	id: string
	values: LegalMentionFormValues
} | null

const EMPTY_LEGAL_FORM: LegalMentionFormValues = { name: '', body: '' }

function LegalMentionSection({
	templates,
	onCreate,
	onUpdate,
	onDelete,
}: LegalMentionSectionProps) {
	const [draft, setDraft] = useState<LegalMentionDraft>(null)
	const [createForm, setCreateForm] =
		useState<LegalMentionFormValues>(EMPTY_LEGAL_FORM)
	const [isSaving, setIsSaving] = useState(false)
	const [isCreating, setIsCreating] = useState(false)

	const handleCreate = async () => {
		if (!createForm.name.trim() || !createForm.body.trim()) return
		setIsCreating(true)
		try {
			await onCreate(createForm)
			setCreateForm(EMPTY_LEGAL_FORM)
		} finally {
			setIsCreating(false)
		}
	}

	const handleSaveDraft = async () => {
		if (!draft) return
		const template = templates.find((t) => t.id === draft.id)
		if (!template) return
		setIsSaving(true)
		try {
			await onUpdate(template, draft.values)
			setDraft(null)
		} finally {
			setIsSaving(false)
		}
	}

	const applyPreset = (presetId: string) => {
		const preset = LEGAL_MENTION_PRESETS.find((p) => p.id === presetId)
		if (!preset) return
		setCreateForm({ name: preset.name, body: preset.body })
	}

	return (
		<SectionCard>
			<SectionHeader
				title={`Mentions légales (${templates.length})`}
				description="Modèles de mentions légales réutilisables dans les documents commerciaux."
				actions={
					<StatusBadge tone="brand">
						<FileText className="mr-1 size-3" />
						Modèles
					</StatusBadge>
				}
			/>
			<div className="flex flex-col gap-4 p-5">
				<div className="flex flex-col gap-3 rounded-lg border bg-muted/30 p-4">
					<div className="flex flex-col gap-1">
						<p className="text-sm font-medium">Nouveau modèle</p>
						<p className="text-xs text-muted-foreground">
							Saisissez manuellement ou choisissez un modèle prédéfini.
						</p>
					</div>

					<div className="flex flex-wrap items-center gap-2">
						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button
									type="button"
									variant="outline"
									size="sm"
									className="gap-2"
								>
									<FileText className="size-3.5" />
									Insérer un modèle
									<ChevronDown className="size-3.5" />
								</Button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="start" className="w-72">
								{LEGAL_MENTION_PRESETS.map((preset) => (
									<DropdownMenuItem
										key={preset.id}
										onClick={() => applyPreset(preset.id)}
									>
										{preset.label}
									</DropdownMenuItem>
								))}
							</DropdownMenuContent>
						</DropdownMenu>
					</div>

					<div className="grid grid-cols-1 gap-3">
						<TextField
							label="Nom du modèle"
							value={createForm.name}
							onChange={(v) => setCreateForm((f) => ({ ...f, name: v }))}
							placeholder="ex. Pénalités de retard"
						/>
						<div className="flex flex-col gap-2">
							<Label htmlFor="legal-create-body">Contenu</Label>
							<Textarea
								id="legal-create-body"
								value={createForm.body}
								onChange={(e) =>
									setCreateForm((f) => ({ ...f, body: e.target.value }))
								}
								rows={4}
								placeholder="Texte de la mention légale…"
							/>
						</div>
					</div>
					<div className="flex justify-end">
						<Button
							type="button"
							onClick={handleCreate}
							disabled={
								isCreating || !createForm.name.trim() || !createForm.body.trim()
							}
							size="sm"
							className="gap-2"
						>
							{isCreating ? (
								<Loader2 className="size-3.5 animate-spin" />
							) : (
								<Plus className="size-3.5" />
							)}
							Ajouter
						</Button>
					</div>
				</div>

				{templates.length === 0 ? (
					<div className="flex flex-col items-center gap-2 py-8 text-center text-sm text-muted-foreground">
						<FileText className="size-8 opacity-30" />
						<p className="font-medium text-foreground">
							Aucune mention légale configurée
						</p>
						<p>Créez votre premier modèle ci-dessus.</p>
					</div>
				) : (
					<div className="flex flex-col divide-y rounded-lg border">
						{templates.map((template) => {
							const isEditing = draft?.id === template.id
							return (
								<div
									key={template.id}
									className="group flex flex-col gap-3 p-4"
								>
									{isEditing ? (
										<>
											<TextField
												label="Nom"
												value={draft.values.name}
												onChange={(v) =>
													setDraft((d) =>
														d ? { ...d, values: { ...d.values, name: v } } : d,
													)
												}
											/>
											<div className="flex flex-col gap-2">
												<Label htmlFor={`legal-edit-body-${template.id}`}>
													Contenu
												</Label>
												<Textarea
													id={`legal-edit-body-${template.id}`}
													value={draft.values.body}
													onChange={(e) =>
														setDraft((d) =>
															d
																? {
																		...d,
																		values: {
																			...d.values,
																			body: e.target.value,
																		},
																	}
																: d,
														)
													}
													rows={5}
												/>
											</div>
											<div className="flex justify-end gap-2">
												<Button
													size="sm"
													variant="ghost"
													onClick={() => setDraft(null)}
												>
													<Undo2 className="size-3.5" />
													Annuler
												</Button>
												<Button
													size="sm"
													onClick={handleSaveDraft}
													disabled={isSaving}
												>
													{isSaving ? (
														<Loader2 className="size-3.5 animate-spin" />
													) : (
														<Save className="size-3.5" />
													)}
													Enregistrer
												</Button>
											</div>
										</>
									) : (
										<div className="flex items-start justify-between gap-3">
											<div className="min-w-0">
												<p className="truncate font-medium">{template.name}</p>
												<p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
													{template.body}
												</p>
											</div>
											<div className="opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
												<DropdownMenu>
													<DropdownMenuTrigger asChild>
														<Button size="icon-sm" variant="ghost">
															<MoreHorizontal />
															<span className="sr-only">Actions</span>
														</Button>
													</DropdownMenuTrigger>
													<DropdownMenuContent align="end">
														<DropdownMenuItem
															onClick={() =>
																setDraft({
																	id: template.id,
																	values: {
																		name: template.name,
																		body: template.body,
																	},
																})
															}
														>
															Modifier
														</DropdownMenuItem>
														<DropdownMenuSeparator />
														<DropdownMenuItem
															variant="destructive"
															onClick={() => onDelete(template)}
														>
															<Trash2 />
															Supprimer
														</DropdownMenuItem>
													</DropdownMenuContent>
												</DropdownMenu>
											</div>
										</div>
									)}
								</div>
							)
						})}
					</div>
				)}
			</div>
		</SectionCard>
	)
}

// ─── Emitter Context Section ──────────────────────────────────────────────────

interface EmitterContextSectionProps {
	contexts: OrganizationContext[]
	onCreate: (values: EmitterContextFormValues) => Promise<unknown>
	onUpdate: (
		context: OrganizationContext,
		values: EmitterContextFormValues,
	) => Promise<unknown>
	onDelete: (context: OrganizationContext) => Promise<unknown>
}

type EmitterContextDraft = {
	id: string
	values: EmitterContextFormValues
} | null

const EMPTY_EMITTER_FORM: EmitterContextFormValues = {
	label: '',
	address_line: '',
	postal_code: '',
	city: '',
	country: '',
	siret: '',
	rcs: '',
	ape: '',
	vat_intracom: '',
	iban: '',
	bic: '',
}

function EmitterContextSection({
	contexts,
	onCreate,
	onUpdate,
	onDelete,
}: EmitterContextSectionProps) {
	const [draft, setDraft] = useState<EmitterContextDraft>(null)
	const [createForm, setCreateForm] =
		useState<EmitterContextFormValues>(EMPTY_EMITTER_FORM)
	const [isSaving, setIsSaving] = useState(false)
	const [isCreating, setIsCreating] = useState(false)

	const handleCreate = async () => {
		if (!createForm.label.trim()) return
		setIsCreating(true)
		try {
			await onCreate(createForm)
			setCreateForm(EMPTY_EMITTER_FORM)
		} finally {
			setIsCreating(false)
		}
	}

	const handleSaveDraft = async () => {
		if (!draft) return
		const context = contexts.find((c) => c.id === draft.id)
		if (!context) return
		setIsSaving(true)
		try {
			await onUpdate(context, draft.values)
			setDraft(null)
		} finally {
			setIsSaving(false)
		}
	}

	return (
		<SectionCard>
			<SectionHeader
				title={`Contextes \u00e9metteurs (${contexts.length})`}
				description="Identit\u00e9s d'\u00e9mission utilis\u00e9es sur les devis et factures (si\u00e8ge, agences, entit\u00e9s\u2026)."
				actions={
					<StatusBadge tone="brand">
						<Landmark className="mr-1 size-3" />
						\u00c9metteurs
					</StatusBadge>
				}
			/>
			<div className="flex flex-col gap-4 p-5">
				<div className="flex flex-col gap-3 rounded-lg border bg-muted/30 p-4">
					<div className="flex flex-col gap-1">
						<p className="text-sm font-medium">Nouveau contexte</p>
						<p className="text-xs text-muted-foreground">
							Seul le nom est obligatoire.
						</p>
					</div>

					<div className="flex flex-col gap-4">
						<fieldset className="flex flex-col gap-3">
							<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
								Nomination
							</legend>
							<TextField
								label="Nom du contexte"
								value={createForm.label}
								onChange={(v) => setCreateForm((f) => ({ ...f, label: v }))}
								placeholder="ex. Si\u00e8ge, Agence Lyon\u2026"
							/>
						</fieldset>

						<fieldset className="flex flex-col gap-3">
							<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
								Adresse
							</legend>
							<div className="grid grid-cols-1 gap-3 md:grid-cols-2">
								<TextField
									label="Adresse"
									value={createForm.address_line}
									onChange={(v) =>
										setCreateForm((f) => ({ ...f, address_line: v }))
									}
									placeholder="12 rue de la Paix"
								/>
								<TextField
									label="Code postal"
									value={createForm.postal_code}
									onChange={(v) =>
										setCreateForm((f) => ({ ...f, postal_code: v }))
									}
									placeholder="75000"
								/>
								<TextField
									label="Ville"
									value={createForm.city}
									onChange={(v) => setCreateForm((f) => ({ ...f, city: v }))}
									placeholder="Paris"
								/>
								<TextField
									label="Pays"
									value={createForm.country}
									onChange={(v) => setCreateForm((f) => ({ ...f, country: v }))}
									placeholder="France"
								/>
							</div>
						</fieldset>

						<fieldset className="flex flex-col gap-3">
							<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
								Identit\u00e9 l\u00e9gale
							</legend>
							<div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-4">
								<TextField
									label="SIRET"
									value={createForm.siret}
									onChange={(v) => setCreateForm((f) => ({ ...f, siret: v }))}
									placeholder="14 chiffres"
								/>
								<TextField
									label="RCS"
									value={createForm.rcs}
									onChange={(v) => setCreateForm((f) => ({ ...f, rcs: v }))}
									placeholder="Ville + n\u00b0 immat."
								/>
								<TextField
									label="APE / NAF"
									value={createForm.ape}
									onChange={(v) => setCreateForm((f) => ({ ...f, ape: v }))}
									placeholder="ex. 4321A"
								/>
								<TextField
									label="TVA intracom"
									value={createForm.vat_intracom}
									onChange={(v) =>
										setCreateForm((f) => ({ ...f, vat_intracom: v }))
									}
									placeholder="ex. FR12345678901"
								/>
							</div>
						</fieldset>

						<fieldset className="flex flex-col gap-3">
							<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
								Banque
							</legend>
							<div className="grid grid-cols-1 gap-3 md:grid-cols-2">
								<TextField
									label="IBAN"
									value={createForm.iban}
									onChange={(v) => setCreateForm((f) => ({ ...f, iban: v }))}
									placeholder="FR76 \u2026"
								/>
								<TextField
									label="BIC"
									value={createForm.bic}
									onChange={(v) => setCreateForm((f) => ({ ...f, bic: v }))}
									placeholder="ex. BNPAFRPP"
								/>
							</div>
						</fieldset>
					</div>

					<div className="flex justify-end">
						<Button
							type="button"
							onClick={handleCreate}
							disabled={isCreating || !createForm.label.trim()}
							size="sm"
							className="gap-2"
						>
							{isCreating ? (
								<Loader2 className="size-3.5 animate-spin" />
							) : (
								<Plus className="size-3.5" />
							)}
							Ajouter
						</Button>
					</div>
				</div>

				{contexts.length === 0 ? (
					<div className="flex flex-col items-center gap-2 py-8 text-center text-sm text-muted-foreground">
						<Landmark className="size-8 opacity-30" />
						<p className="font-medium text-foreground">
							Aucun contexte \u00e9metteur configur\u00e9
						</p>
						<p>Cr\u00e9ez votre premier contexte ci-dessus.</p>
					</div>
				) : (
					<div className="flex flex-col divide-y rounded-lg border">
						{contexts.map((context) => {
							const isEditing = draft?.id === context.id
							const summary = [context.city, context.siret]
								.filter(Boolean)
								.join(' \u00b7 ')
							return (
								<div key={context.id} className="group flex flex-col gap-3 p-4">
									{isEditing ? (
										<>
											<fieldset className="flex flex-col gap-3">
												<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
													Nomination
												</legend>
												<TextField
													label="Nom du contexte"
													value={draft.values.label}
													onChange={(v) =>
														setDraft((d) =>
															d
																? { ...d, values: { ...d.values, label: v } }
																: d,
														)
													}
												/>
											</fieldset>

											<fieldset className="flex flex-col gap-3">
												<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
													Adresse
												</legend>
												<div className="grid grid-cols-1 gap-3 md:grid-cols-2">
													<TextField
														label="Adresse"
														value={draft.values.address_line}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? {
																			...d,
																			values: { ...d.values, address_line: v },
																		}
																	: d,
															)
														}
													/>
													<TextField
														label="Code postal"
														value={draft.values.postal_code}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? {
																			...d,
																			values: { ...d.values, postal_code: v },
																		}
																	: d,
															)
														}
													/>
													<TextField
														label="Ville"
														value={draft.values.city}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? { ...d, values: { ...d.values, city: v } }
																	: d,
															)
														}
													/>
													<TextField
														label="Pays"
														value={draft.values.country}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? {
																			...d,
																			values: { ...d.values, country: v },
																		}
																	: d,
															)
														}
													/>
												</div>
											</fieldset>

											<fieldset className="flex flex-col gap-3">
												<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
													Identit\u00e9 l\u00e9gale
												</legend>
												<div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-4">
													<TextField
														label="SIRET"
														value={draft.values.siret}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? { ...d, values: { ...d.values, siret: v } }
																	: d,
															)
														}
													/>
													<TextField
														label="RCS"
														value={draft.values.rcs}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? { ...d, values: { ...d.values, rcs: v } }
																	: d,
															)
														}
													/>
													<TextField
														label="APE / NAF"
														value={draft.values.ape}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? { ...d, values: { ...d.values, ape: v } }
																	: d,
															)
														}
													/>
													<TextField
														label="TVA intracom"
														value={draft.values.vat_intracom}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? {
																			...d,
																			values: { ...d.values, vat_intracom: v },
																		}
																	: d,
															)
														}
													/>
												</div>
											</fieldset>

											<fieldset className="flex flex-col gap-3">
												<legend className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
													Banque
												</legend>
												<div className="grid grid-cols-1 gap-3 md:grid-cols-2">
													<TextField
														label="IBAN"
														value={draft.values.iban}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? { ...d, values: { ...d.values, iban: v } }
																	: d,
															)
														}
													/>
													<TextField
														label="BIC"
														value={draft.values.bic}
														onChange={(v) =>
															setDraft((d) =>
																d
																	? { ...d, values: { ...d.values, bic: v } }
																	: d,
															)
														}
													/>
												</div>
											</fieldset>

											<div className="flex justify-end gap-2">
												<Button
													size="sm"
													variant="ghost"
													onClick={() => setDraft(null)}
												>
													<Undo2 className="size-3.5" />
													Annuler
												</Button>
												<Button
													size="sm"
													onClick={handleSaveDraft}
													disabled={isSaving}
												>
													{isSaving ? (
														<Loader2 className="size-3.5 animate-spin" />
													) : (
														<Save className="size-3.5" />
													)}
													Enregistrer
												</Button>
											</div>
										</>
									) : (
										<div className="flex items-start justify-between gap-3">
											<div className="min-w-0">
												<p className="truncate font-medium">{context.label}</p>
												{summary ? (
													<p className="mt-1 truncate text-sm text-muted-foreground">
														{summary}
													</p>
												) : null}
											</div>
											<div className="opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
												<DropdownMenu>
													<DropdownMenuTrigger asChild>
														<Button size="icon-sm" variant="ghost">
															<MoreHorizontal />
															<span className="sr-only">Actions</span>
														</Button>
													</DropdownMenuTrigger>
													<DropdownMenuContent align="end">
														<DropdownMenuItem
															onClick={() =>
																setDraft({
																	id: context.id,
																	values: {
																		label: context.label,
																		address_line: context.address_line ?? '',
																		postal_code: context.postal_code ?? '',
																		city: context.city ?? '',
																		country: context.country ?? '',
																		siret: context.siret ?? '',
																		rcs: context.rcs ?? '',
																		ape: context.ape ?? '',
																		vat_intracom: context.vat_intracom ?? '',
																		iban: context.iban ?? '',
																		bic: context.bic ?? '',
																	},
																})
															}
														>
															Modifier
														</DropdownMenuItem>
														<DropdownMenuSeparator />
														<DropdownMenuItem
															variant="destructive"
															onClick={() => onDelete(context)}
														>
															<Trash2 />
															Supprimer
														</DropdownMenuItem>
													</DropdownMenuContent>
												</DropdownMenu>
											</div>
										</div>
									)}
								</div>
							)
						})}
					</div>
				)}
			</div>
		</SectionCard>
	)
}
