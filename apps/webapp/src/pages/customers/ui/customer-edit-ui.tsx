import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	Image,
	MapPin,
	Pencil,
	Plus,
	Trash2,
	Upload,
} from 'lucide-react'
import { FloatingActionBar } from '#/components/floating-action-bar'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	EntityAvatar,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
} from '#/components/ui/surface'
import type { Customer, Property } from '#/hooks/use-customers'
import {
	type CustomerFormValues,
	customerInitials,
	type PropertyFormValues,
} from '#/pages/customers/types'

interface CustomerEditUIProps {
	customer: Customer
	form: CustomerFormValues
	isDirty: boolean
	changedKeys: (keyof CustomerFormValues)[]
	isSaving: boolean
	properties: Property[]
	propertiesError: string | null
	isPropertiesLoading: boolean
	propertyDraft: PropertyFormValues
	editingPropertyId: string | null
	isPropertySaving: boolean
	isUploadingPhoto: boolean
	deletingPropertyId: string | null
	photoPreviewUrl: string | null
	onChange: (patch: Partial<CustomerFormValues>) => void
	onReset: () => void
	onSave: () => void
	onPropertyChange: (patch: Partial<PropertyFormValues>) => void
	onPropertyEdit: (property: Property) => void
	onPropertyCancel: () => void
	onPropertySubmit: () => void
	onPropertyDelete: (property: Property) => void
	onPropertyPhotoChange: (file: File) => void
	onRetryProperties: () => void
}

export function CustomerEditUI({
	customer,
	form,
	isDirty,
	changedKeys,
	isSaving,
	properties,
	propertiesError,
	isPropertiesLoading,
	propertyDraft,
	editingPropertyId,
	isPropertySaving,
	isUploadingPhoto,
	deletingPropertyId,
	photoPreviewUrl,
	onChange,
	onReset,
	onSave,
	onPropertyChange,
	onPropertyEdit,
	onPropertyCancel,
	onPropertySubmit,
	onPropertyDelete,
	onPropertyPhotoChange,
	onRetryProperties,
}: CustomerEditUIProps) {
	const displayName = `${form.firstName} ${form.lastName}`.trim()
	const canSubmitProperty =
		propertyDraft.label.trim() &&
		propertyDraft.street.trim() &&
		propertyDraft.zip.trim() &&
		propertyDraft.city.trim()

	return (
		<PageShell className="pb-24 md:pb-28">
			<div>
				<Link
					to="/customers"
					className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
				>
					<ArrowLeft className="size-4" />
					Retour aux clients
				</Link>
			</div>

			<PageHeader
				title={displayName || 'Nouveau client'}
				description={
					<span className="font-mono text-xs">id: {customer.id}</span>
				}
				className="items-center sm:items-center sm:justify-start"
				eyebrow="Client"
				leading={
					<EntityAvatar tone="brand" size="lg">
						{customerInitials(customer)}
					</EntityAvatar>
				}
			/>

			<div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
				<Section
					title="Identité"
					description="Informations principales du client"
					className="lg:col-span-2"
				>
					<Field
						label="Prénom"
						name="firstName"
						value={form.firstName}
						onChange={(v) => onChange({ firstName: v })}
						changed={changedKeys.includes('firstName')}
					/>
					<Field
						label="Nom"
						name="lastName"
						value={form.lastName}
						onChange={(v) => onChange({ lastName: v })}
						changed={changedKeys.includes('lastName')}
					/>
				</Section>

				<Section title="Coordonnées" description="Email et téléphone">
					<Field
						label="Email"
						name="email"
						type="email"
						value={form.email}
						onChange={(v) => onChange({ email: v })}
						changed={changedKeys.includes('email')}
					/>
					<Field
						label="Téléphone"
						name="phone"
						value={form.phone}
						onChange={(v) => onChange({ phone: v })}
						changed={changedKeys.includes('phone')}
					/>
				</Section>

				<Section
					title="Sites et adresses"
					description="Adresses associées à ce client"
					className="lg:col-span-3"
				>
					{propertiesError ? (
						<div className="mb-4 flex flex-col gap-3 rounded-lg border border-destructive/30 bg-destructive-soft p-4 text-destructive sm:flex-row sm:items-center sm:justify-between">
							<div className="flex items-center gap-2">
								<AlertCircle className="size-4" />
								<p className="text-sm font-medium">{propertiesError}</p>
							</div>
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={onRetryProperties}
							>
								Réessayer
							</Button>
						</div>
					) : null}

					{isPropertiesLoading ? (
						<div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
							Chargement des sites…
						</div>
					) : properties.length === 0 ? (
						<div className="rounded-lg border border-dashed p-8 text-center">
							<MapPin className="mx-auto size-6 text-muted-foreground" />
							<p className="mt-2 font-medium">Aucun site renseigné</p>
							<p className="mt-1 text-sm text-muted-foreground">
								Ajoutez une adresse pour ce client.
							</p>
						</div>
					) : (
						<div className="grid gap-3 lg:grid-cols-2">
							{properties.map((property) => (
								<PropertyCard
									key={property.id}
									property={property}
									isDeleting={deletingPropertyId === property.id}
									onEdit={() => onPropertyEdit(property)}
									onDelete={() => onPropertyDelete(property)}
								/>
							))}
						</div>
					)}

					<div className="mt-5 rounded-lg border bg-muted/20">
						<div className="flex items-center justify-between gap-4 border-b px-4 py-3">
							<div>
								<p className="font-medium">
									{editingPropertyId ? 'Modifier le site' : 'Ajouter un site'}
								</p>
								<p className="text-xs text-muted-foreground">
									Les champs adresse sont indépendants du client.
								</p>
							</div>
							{editingPropertyId ? (
								<Button
									type="button"
									variant="ghost"
									size="sm"
									onClick={onPropertyCancel}
								>
									Annuler
								</Button>
							) : null}
						</div>
						<div className="grid gap-4 p-4 md:grid-cols-2">
							<Field
								label="Libellé"
								name="property-label"
								value={propertyDraft.label}
								onChange={(v) => onPropertyChange({ label: v })}
							/>
							<Field
								label="Ville"
								name="property-city"
								value={propertyDraft.city}
								onChange={(v) => onPropertyChange({ city: v })}
							/>
							<Field
								label="Rue"
								name="property-street"
								value={propertyDraft.street}
								onChange={(v) => onPropertyChange({ street: v })}
							/>
							<Field
								label="Code postal"
								name="property-zip"
								value={propertyDraft.zip}
								onChange={(v) => onPropertyChange({ zip: v })}
							/>
						</div>
						<div className="grid gap-4 border-t p-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
							<div className="flex flex-col gap-2">
								<Label htmlFor="property-photo">Photo du site</Label>
								<div className="flex flex-col gap-3 sm:flex-row sm:items-center">
									<label
										htmlFor="property-photo"
										className="inline-flex h-9 cursor-pointer items-center justify-center gap-2 rounded-lg border border-primary/40 bg-card px-4 text-sm font-medium text-primary shadow-xs hover:bg-brand-soft"
									>
										<Upload className="size-4" />
										{isUploadingPhoto ? 'Téléversement…' : 'Téléverser'}
									</label>
									<input
										id="property-photo"
										type="file"
										accept="image/*"
										className="sr-only"
										disabled={isUploadingPhoto}
										onChange={(event) => {
											const file = event.target.files?.[0]
											if (file) onPropertyPhotoChange(file)
											event.currentTarget.value = ''
										}}
									/>
									{propertyDraft.photoKey ? (
										<span className="min-w-0 truncate font-mono text-xs text-muted-foreground">
											{propertyDraft.photoKey}
										</span>
									) : null}
								</div>
							</div>

							{photoPreviewUrl ? (
								<img
									src={photoPreviewUrl}
									alt="Aperçu du site"
									className="h-24 w-36 rounded-lg border object-cover"
								/>
							) : null}
						</div>
						<div className="flex justify-end border-t p-4">
							<Button
								type="button"
								disabled={!canSubmitProperty || isPropertySaving}
								onClick={onPropertySubmit}
							>
								<Plus />
								{editingPropertyId ? 'Enregistrer le site' : 'Ajouter le site'}
							</Button>
						</div>
					</div>
				</Section>
			</div>

			<FloatingActionBar
				show={isDirty}
				message={
					changedKeys.length === 1
						? '1 modification non enregistrée'
						: `${changedKeys.length} modifications non enregistrées`
				}
				confirmLabel="Enregistrer"
				cancelLabel="Annuler"
				onCancel={onReset}
				onConfirm={onSave}
				isLoading={isSaving}
			/>
		</PageShell>
	)
}

export namespace CustomerEditUI {
	export function Loading() {
		return (
			<PageShell>
				<SectionCard className="flex items-center justify-center p-12 text-sm text-muted-foreground">
					Chargement…
				</SectionCard>
			</PageShell>
		)
	}
}

interface SectionProps {
	title: string
	description?: string
	className?: string
	children: React.ReactNode
}

function Section({
	title,
	description,
	className = '',
	children,
}: SectionProps) {
	return (
		<SectionCard className={className}>
			<SectionHeader title={title} description={description} />
			<div className="flex flex-col gap-4 p-5">{children}</div>
		</SectionCard>
	)
}

interface FieldProps {
	label: string
	name: string
	value: string
	onChange: (v: string) => void
	type?: string
	changed?: boolean
}

function Field({
	label,
	name,
	value,
	onChange,
	type = 'text',
	changed,
}: FieldProps) {
	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={name}>
				{label}
				{changed ? <Dot /> : null}
			</Label>
			<Input
				id={name}
				name={name}
				type={type}
				value={value}
				onChange={(e) => onChange(e.target.value)}
			/>
		</div>
	)
}

function Dot() {
	return (
		<span
			role="img"
			aria-label="modifié"
			className="ml-1.5 inline-block size-1.5 rounded-full bg-primary align-middle"
		/>
	)
}

interface PropertyCardProps {
	property: Property
	isDeleting: boolean
	onEdit: () => void
	onDelete: () => void
}

function PropertyCard({
	property,
	isDeleting,
	onEdit,
	onDelete,
}: PropertyCardProps) {
	return (
		<div className="flex gap-4 rounded-lg border bg-card p-4">
			<div className="flex size-16 shrink-0 items-center justify-center overflow-hidden rounded-lg border bg-muted">
				{property.photo_key ? (
					<Image className="size-6 text-muted-foreground" />
				) : (
					<MapPin className="size-6 text-muted-foreground" />
				)}
			</div>
			<div className="min-w-0 flex-1">
				<div className="flex items-start justify-between gap-3">
					<div className="min-w-0">
						<p className="truncate font-semibold">{property.label}</p>
						<p className="mt-1 text-sm text-muted-foreground">
							{property.street}
						</p>
						<p className="text-sm text-muted-foreground">
							{property.zip} {property.city}
						</p>
					</div>
					<div className="flex shrink-0 gap-1">
						<Button
							type="button"
							variant="ghost"
							size="icon-sm"
							onClick={onEdit}
						>
							<Pencil />
							<span className="sr-only">Modifier</span>
						</Button>
						<Button
							type="button"
							variant="ghost"
							size="icon-sm"
							disabled={isDeleting}
							onClick={onDelete}
						>
							<Trash2 />
							<span className="sr-only">Supprimer</span>
						</Button>
					</div>
				</div>
				{property.photo_key ? (
					<p className="mt-2 truncate font-mono text-xs text-muted-foreground">
						photo: {property.photo_key}
					</p>
				) : null}
			</div>
		</div>
	)
}
