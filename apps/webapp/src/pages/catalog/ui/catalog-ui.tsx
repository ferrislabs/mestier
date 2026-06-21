import {
	Boxes,
	BriefcaseBusiness,
	Loader2,
	MoreHorizontal,
	Package,
	Plus,
	Save,
	Search,
	Trash2,
	Undo2,
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
	Sheet,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle,
} from '#/components/ui/sheet'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import type { ProductCatalogFormValues } from '#/hooks/use-catalog-items'
import type {
	Product,
	ServiceRate,
	ServiceRateUnit,
} from '#/hooks/use-reference-catalog'
import type { ServiceRateFormValues } from '#/pages/settings/types'

type CatalogTab = 'products' | 'services'

interface FormBinding<T> {
	values: T
	isPending: boolean
	onChange: (patch: Partial<T>) => void
	onSubmit: () => void | Promise<void>
}

type Draft =
	| { tab: 'products'; id: string; values: ProductCatalogFormValues }
	| { tab: 'services'; id: string; values: ServiceRateFormValues }
	| null

interface CatalogUIProps {
	organizationName: string
	isLoading: boolean
	error: string | null
	serviceRates: ServiceRate[]
	products: Product[]
	serviceRateForm: FormBinding<ServiceRateFormValues>
	productForm: FormBinding<ProductCatalogFormValues>
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
}

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

export function CatalogUI({
	organizationName,
	isLoading,
	error,
	serviceRates,
	products,
	serviceRateForm,
	productForm,
	onUpdateServiceRate,
	onDeleteServiceRate,
	onUpdateProduct,
	onDeleteProduct,
}: CatalogUIProps) {
	const [activeTab, setActiveTab] = useState<CatalogTab>('products')
	const [createOpen, setCreateOpen] = useState(false)
	const [createMode, setCreateMode] = useState<CatalogTab>('products')
	const [search, setSearch] = useState('')
	const [draft, setDraft] = useState<Draft>(null)
	const [isSaving, setIsSaving] = useState(false)

	const normalizedSearch = search.trim().toLowerCase()
	const filteredProducts = useMemo(() => {
		return products.filter((product) => {
			return (
				product.name.toLowerCase().includes(normalizedSearch) ||
				(product.sku ?? '').toLowerCase().includes(normalizedSearch) ||
				(product.description ?? '').toLowerCase().includes(normalizedSearch)
			)
		})
	}, [normalizedSearch, products])

	const filteredServices = useMemo(() => {
		return serviceRates.filter((serviceRate) =>
			serviceRate.label.toLowerCase().includes(normalizedSearch),
		)
	}, [normalizedSearch, serviceRates])

	const catalogValueCents = products.reduce(
		(sum, product) => sum + product.unit_price_cents,
		0,
	)
	const defaultProductPrice = products.length
		? Math.round(catalogValueCents / products.length)
		: 0

	const handleSaveDraft = async () => {
		if (!draft) return
		setIsSaving(true)
		try {
			if (draft.tab === 'products') {
				const product = products.find((item) => item.id === draft.id)
				if (product) await onUpdateProduct(product, draft.values)
			}
			if (draft.tab === 'services') {
				const serviceRate = serviceRates.find((item) => item.id === draft.id)
				if (serviceRate) await onUpdateServiceRate(serviceRate, draft.values)
			}
			setDraft(null)
		} finally {
			setIsSaving(false)
		}
	}

	const openCreate = (mode: CatalogTab = activeTab) => {
		setCreateMode(mode)
		setCreateOpen(true)
	}

	const handleCreateSubmit = async () => {
		if (createMode === 'products') {
			await productForm.onSubmit()
			setCreateOpen(false)
			return
		}
		await serviceRateForm.onSubmit()
		setCreateOpen(false)
	}

	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Catalogue"
				description="Centralisez les produits et prestations utilisés pour composer les devis et les prochaines pièces commerciales."
				actions={
					<div className="flex flex-col gap-2 sm:flex-row">
						<Button
							type="button"
							variant="outline"
							onClick={() => openCreate('services')}
						>
							<BriefcaseBusiness />
							Nouveau service
						</Button>
						<Button type="button" onClick={() => openCreate('products')}>
							<Plus />
							Nouveau produit
						</Button>
					</div>
				}
			/>

			<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
				<MetricCard
					label="Produits"
					value={products.length}
					hint="Articles vendables"
					icon={<Boxes className="size-4" />}
				/>
				<MetricCard
					label="Services"
					value={serviceRates.length}
					hint="Prestations chiffrables"
					icon={<BriefcaseBusiness className="size-4" />}
				/>
				<MetricCard
					label="Prix produit moyen"
					value={formatMoney(defaultProductPrice)}
					hint="Base de chiffrage"
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
					<SegmentButton
						active={activeTab === 'products'}
						icon={<Boxes className="size-4" />}
						label="Produits"
						onClick={() => {
							setActiveTab('products')
							setDraft(null)
						}}
					/>
					<SegmentButton
						active={activeTab === 'services'}
						icon={<BriefcaseBusiness className="size-4" />}
						label="Services"
						onClick={() => {
							setActiveTab('services')
							setDraft(null)
						}}
					/>
				</div>
				<div className="relative w-full lg:w-96">
					<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
					<Input
						type="search"
						value={search}
						onChange={(event) => setSearch(event.target.value)}
						placeholder="Rechercher un produit, une référence ou un service"
						className="pl-9"
					/>
				</div>
			</div>

			{isLoading ? (
				<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
					<Loader2 className="size-5 animate-spin" />
					Chargement du catalogue…
				</SectionCard>
			) : activeTab === 'products' ? (
				<ProductList
					products={filteredProducts}
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
								description: product.description ?? '',
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
					onCreate={() => openCreate('products')}
				/>
			) : (
				<ServiceList
					serviceRates={filteredServices}
					draft={draft}
					isSaving={isSaving}
					onEdit={(serviceRate) =>
						setDraft({
							tab: 'services',
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
							current?.tab === 'services' ? { ...current, values } : current,
						)
					}
					onCancel={() => setDraft(null)}
					onSave={handleSaveDraft}
					onDelete={onDeleteServiceRate}
					onCreate={() => openCreate('services')}
				/>
			)}

			<Sheet open={createOpen} onOpenChange={setCreateOpen}>
				<SheetContent className="w-full overflow-y-auto sm:max-w-xl">
					<SheetHeader className="border-b">
						<SheetTitle>
							{createMode === 'products'
								? 'Nouveau produit'
								: 'Nouveau service'}
						</SheetTitle>
						<SheetDescription>
							{createMode === 'products'
								? 'Ajoutez un article vendable qui pourra être inséré dans les devis.'
								: 'Ajoutez une prestation réutilisable avec son unité de facturation.'}
						</SheetDescription>
					</SheetHeader>
					<div className="flex-1 overflow-y-auto p-4">
						{createMode === 'products' ? (
							<ProductCreateFields form={productForm} />
						) : (
							<ServiceCreateFields form={serviceRateForm} />
						)}
					</div>
					<SheetFooter className="border-t sm:flex-row sm:justify-end">
						<Button
							type="button"
							variant="ghost"
							onClick={() => setCreateOpen(false)}
						>
							Annuler
						</Button>
						<Button
							type="button"
							onClick={handleCreateSubmit}
							disabled={
								createMode === 'products'
									? productForm.isPending
									: serviceRateForm.isPending
							}
						>
							{(
								createMode === 'products'
									? productForm.isPending
									: serviceRateForm.isPending
							) ? (
								<Loader2 className="animate-spin" />
							) : (
								<Plus />
							)}
							Ajouter
						</Button>
					</SheetFooter>
				</SheetContent>
			</Sheet>
		</PageShell>
	)
}

function SegmentButton({
	active,
	icon,
	label,
	onClick,
}: {
	active: boolean
	icon: React.ReactNode
	label: string
	onClick: () => void
}) {
	return (
		<Button
			type="button"
			variant={active ? 'default' : 'outline'}
			onClick={onClick}
		>
			{icon}
			{label}
		</Button>
	)
}

function ProductCreateFields({
	form,
}: {
	form: FormBinding<ProductCatalogFormValues>
}) {
	return (
		<div className="space-y-4">
			<TextField
				label="Produit"
				value={form.values.name}
				onChange={(name) => form.onChange({ name })}
			/>
			<TextField
				label="Référence"
				value={form.values.sku}
				onChange={(sku) => form.onChange({ sku })}
				placeholder="Optionnel"
			/>
			<div className="grid gap-4 sm:grid-cols-2">
				<UnitField
					value={form.values.unit}
					onChange={(unit) => form.onChange({ unit })}
					product
				/>
				<TextField
					label="Prix"
					value={form.values.unitPrice}
					onChange={(unitPrice) => form.onChange({ unitPrice })}
					inputMode="decimal"
					suffix={PRODUCT_UNIT_LABELS[form.values.unit]}
				/>
			</div>
			<div className="flex flex-col gap-2">
				<Label>Description</Label>
				<Textarea
					value={form.values.description}
					onChange={(event) =>
						form.onChange({ description: event.target.value })
					}
					placeholder="Optionnel"
				/>
			</div>
		</div>
	)
}

function ServiceCreateFields({
	form,
}: {
	form: FormBinding<ServiceRateFormValues>
}) {
	return (
		<div className="space-y-4">
			<TextField
				label="Libellé"
				value={form.values.label}
				onChange={(label) => form.onChange({ label })}
			/>
			<div className="grid gap-4 sm:grid-cols-2">
				<UnitField
					value={form.values.unit}
					onChange={(unit) => form.onChange({ unit })}
				/>
				<TextField
					label="Tarif"
					value={form.values.rate}
					onChange={(rate) => form.onChange({ rate })}
					inputMode="decimal"
					suffix={UNIT_LABELS[form.values.unit]}
				/>
			</div>
		</div>
	)
}

function ProductList({
	products,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancel,
	onSave,
	onDelete,
	onCreate,
}: {
	products: Product[]
	draft: Draft
	isSaving: boolean
	onEdit: (product: Product) => void
	onDraftChange: (values: ProductCatalogFormValues) => void
	onCancel: () => void
	onSave: () => void
	onDelete: (product: Product) => Promise<unknown>
	onCreate: () => void
}) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Produits (${products.length})`}
				description="Articles vendables avec référence, unité, prix et description préremplie."
			/>
			{products.length === 0 ? (
				<EmptyState
					title="Aucun produit trouvé"
					description="Ajoutez un produit pour l’insérer rapidement dans un devis."
					actionLabel="Nouveau produit"
					onAction={onCreate}
				/>
			) : (
				<ul className="divide-y">
					{products.map((product) => {
						const isEditing =
							draft?.tab === 'products' && draft.id === product.id
						return (
							<li
								key={product.id}
								className="group grid gap-4 px-5 py-4 transition hover:bg-muted/35 hover:shadow-xs lg:grid-cols-[minmax(0,1fr)_140px_140px_140px_40px] lg:items-center"
							>
								{isEditing ? (
									<ProductDraftFields
										values={draft.values}
										onChange={onDraftChange}
									/>
								) : (
									<>
										<div className="min-w-0">
											<div className="flex min-w-0 items-center gap-2">
												<p className="truncate font-semibold">{product.name}</p>
												{product.sku ? (
													<StatusBadge tone="brand">{product.sku}</StatusBadge>
												) : null}
											</div>
											<p className="mt-1 truncate text-sm text-muted-foreground">
												{product.description || 'Aucune description'}
											</p>
										</div>
										<StatusBadge tone="brand">
											{productUnitLabel(product.unit)}
										</StatusBadge>
										<p className="font-semibold tabular-nums">
											{formatMoney(product.unit_price_cents)}
										</p>
										<p className="truncate font-mono text-xs text-muted-foreground">
											{product.id}
										</p>
									</>
								)}
								<RowActions
									isEditing={isEditing}
									isSaving={isSaving}
									onEdit={() => onEdit(product)}
									onCancel={onCancel}
									onSave={onSave}
									onDelete={() => onDelete(product)}
								/>
							</li>
						)
					})}
				</ul>
			)}
		</SectionCard>
	)
}

function ProductDraftFields({
	values,
	onChange,
}: {
	values: ProductCatalogFormValues
	onChange: (values: ProductCatalogFormValues) => void
}) {
	return (
		<>
			<div className="grid gap-3 md:grid-cols-2 lg:col-span-2">
				<Input
					value={values.name}
					onChange={(event) =>
						onChange({ ...values, name: event.target.value })
					}
				/>
				<Input
					value={values.sku}
					onChange={(event) => onChange({ ...values, sku: event.target.value })}
					placeholder="Référence"
				/>
				<Textarea
					value={values.description}
					onChange={(event) =>
						onChange({ ...values, description: event.target.value })
					}
					placeholder="Description"
					className="md:col-span-2"
				/>
			</div>
			<UnitField
				value={values.unit}
				onChange={(unit) => onChange({ ...values, unit })}
				product
			/>
			<Input
				value={values.unitPrice}
				onChange={(event) =>
					onChange({ ...values, unitPrice: event.target.value })
				}
				inputMode="decimal"
			/>
		</>
	)
}

function ServiceList({
	serviceRates,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancel,
	onSave,
	onDelete,
	onCreate,
}: {
	serviceRates: ServiceRate[]
	draft: Draft
	isSaving: boolean
	onEdit: (serviceRate: ServiceRate) => void
	onDraftChange: (values: ServiceRateFormValues) => void
	onCancel: () => void
	onSave: () => void
	onDelete: (serviceRate: ServiceRate) => Promise<unknown>
	onCreate: () => void
}) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Services (${serviceRates.length})`}
				description="Prestations réutilisables dans les devis et futures pièces commerciales."
			/>
			{serviceRates.length === 0 ? (
				<EmptyState
					title="Aucun service trouvé"
					description="Ajoutez une prestation pour accélérer la saisie des devis."
					actionLabel="Nouveau service"
					onAction={onCreate}
				/>
			) : (
				<ul className="divide-y">
					{serviceRates.map((serviceRate) => {
						const isEditing =
							draft?.tab === 'services' && draft.id === serviceRate.id
						return (
							<li
								key={serviceRate.id}
								className="group grid gap-4 px-5 py-4 transition hover:bg-muted/35 hover:shadow-xs lg:grid-cols-[minmax(0,1fr)_140px_140px_40px] lg:items-center"
							>
								{isEditing ? (
									<>
										<Input
											value={draft.values.label}
											onChange={(event) =>
												onDraftChange({
													...draft.values,
													label: event.target.value,
												})
											}
										/>
										<UnitField
											value={draft.values.unit}
											onChange={(unit) =>
												onDraftChange({ ...draft.values, unit })
											}
										/>
										<Input
											value={draft.values.rate}
											onChange={(event) =>
												onDraftChange({
													...draft.values,
													rate: event.target.value,
												})
											}
											inputMode="decimal"
										/>
									</>
								) : (
									<>
										<div className="min-w-0">
											<p className="truncate font-semibold">
												{serviceRate.label}
											</p>
											<p className="mt-1 truncate font-mono text-xs text-muted-foreground">
												{serviceRate.id}
											</p>
										</div>
										<StatusBadge tone="brand">
											{serviceUnitLabel(serviceRate.unit)}
										</StatusBadge>
										<p className="font-semibold tabular-nums">
											{formatMoney(serviceRate.rate_cents)}
										</p>
									</>
								)}
								<RowActions
									isEditing={isEditing}
									isSaving={isSaving}
									onEdit={() => onEdit(serviceRate)}
									onCancel={onCancel}
									onSave={onSave}
									onDelete={() => onDelete(serviceRate)}
								/>
							</li>
						)
					})}
				</ul>
			)}
		</SectionCard>
	)
}

function UnitField({
	value,
	onChange,
	product,
}: {
	value: ServiceRateUnit
	onChange: (unit: ServiceRateUnit) => void
	product?: boolean
}) {
	return (
		<div className="flex flex-col gap-2">
			<Label>Unité</Label>
			<Select
				value={value}
				onValueChange={(unit) => onChange(unit as ServiceRateUnit)}
			>
				<SelectTrigger className="w-full">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					{product ? <SelectItem value="HOUR">Unité</SelectItem> : null}
					{!product ? <SelectItem value="HOUR">Heure</SelectItem> : null}
					<SelectItem value="ML">Mètre linéaire</SelectItem>
					<SelectItem value="M2">Mètre carré</SelectItem>
				</SelectContent>
			</Select>
		</div>
	)
}

function RowActions({
	isEditing,
	isSaving,
	onEdit,
	onCancel,
	onSave,
	onDelete,
}: {
	isEditing: boolean
	isSaving: boolean
	onEdit: () => void
	onCancel: () => void
	onSave: () => void
	onDelete: () => void
}) {
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
					className={suffix ? 'pr-20' : undefined}
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

function EmptyState({
	title,
	description,
	actionLabel,
	onAction,
}: {
	title: string
	description: string
	actionLabel: string
	onAction: () => void
}) {
	return (
		<div className="flex min-h-52 flex-col items-center justify-center gap-3 p-8 text-center">
			<div className="flex size-12 items-center justify-center rounded-lg bg-muted text-muted-foreground">
				<Package className="size-5" />
			</div>
			<div>
				<p className="font-medium">{title}</p>
				<p className="mt-1 max-w-sm text-sm text-muted-foreground">
					{description}
				</p>
			</div>
			<Button type="button" onClick={onAction}>
				<Plus />
				{actionLabel}
			</Button>
		</div>
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

function productUnitLabel(unit: ServiceRateUnit): string {
	if (unit === 'HOUR') return 'unité'
	if (unit === 'ML') return 'mètre linéaire'
	return 'mètre carré'
}

function serviceUnitLabel(unit: ServiceRateUnit): string {
	if (unit === 'HOUR') return 'heure'
	if (unit === 'ML') return 'mètre linéaire'
	return 'mètre carré'
}
