import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ArrowLeft,
	Image,
	LayoutPanelTop,
	Mail,
	Pencil,
	Phone,
	Plus,
	Star,
	Trash2,
	Upload,
	UserCheck,
	UserRound,
} from 'lucide-react'
import { FloatingActionBar } from '#/components/floating-action-bar'
import { Button } from '#/components/ui/button'
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
	EntityAvatar,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Switch } from '#/components/ui/switch'
import type {
	Customer,
	CustomerContact,
	CustomerContext,
	CustomerPipelineStage,
	CustomerStatus,
} from '#/hooks/use-customers'
import {
	type CustomerContactFormValues,
	type CustomerContextFormValues,
	type CustomerFormValues,
	customerInitials,
	customerPipelineStageLabel,
	customerStatusLabel,
} from '#/pages/customers/types'

interface CustomerEditUIProps {
	customer: Customer
	form: CustomerFormValues
	isDirty: boolean
	changedKeys: (keyof CustomerFormValues)[]
	isSaving: boolean
	customerContacts: CustomerContact[]
	customerContactsError: string | null
	isCustomerContactsLoading: boolean
	customerContactDraft: CustomerContactFormValues
	editingCustomerContactId: string | null
	isCustomerContactSaving: boolean
	deletingCustomerContactId: string | null
	customerContexts: CustomerContext[]
	customerContextsError: string | null
	isCustomerContextsLoading: boolean
	customerContextDraft: CustomerContextFormValues
	editingCustomerContextId: string | null
	isCustomerContextSaving: boolean
	isUploadingPhoto: boolean
	deletingCustomerContextId: string | null
	photoPreviewUrl: string | null
	onChange: (patch: Partial<CustomerFormValues>) => void
	onReset: () => void
	onSave: () => void
	onPromoteToClient: () => void
	onCustomerContactChange: (patch: Partial<CustomerContactFormValues>) => void
	onCustomerContactEdit: (customerContact: CustomerContact) => void
	onCustomerContactCancel: () => void
	onCustomerContactSubmit: () => void
	onCustomerContactDelete: (customerContact: CustomerContact) => void
	onCustomerContextChange: (patch: Partial<CustomerContextFormValues>) => void
	onCustomerContextEdit: (customerContext: CustomerContext) => void
	onCustomerContextCancel: () => void
	onCustomerContextSubmit: () => void
	onCustomerContextDelete: (customerContext: CustomerContext) => void
	onCustomerContextPhotoChange: (file: File) => void
	onRetryCustomerContacts: () => void
	onRetryCustomerContexts: () => void
}

const CUSTOMER_PIPELINE_STAGE_OPTIONS: CustomerPipelineStage[] = [
	'NEW',
	'CONTACTED',
	'QUALIFIED',
	'QUOTE_SENT',
	'WON',
	'LOST',
]

export function CustomerEditUI({
	customer,
	form,
	isDirty,
	changedKeys,
	isSaving,
	customerContacts,
	customerContactsError,
	isCustomerContactsLoading,
	customerContactDraft,
	editingCustomerContactId,
	isCustomerContactSaving,
	deletingCustomerContactId,
	customerContexts,
	customerContextsError,
	isCustomerContextsLoading,
	customerContextDraft,
	editingCustomerContextId,
	isCustomerContextSaving,
	isUploadingPhoto,
	deletingCustomerContextId,
	photoPreviewUrl,
	onChange,
	onReset,
	onSave,
	onPromoteToClient,
	onCustomerContactChange,
	onCustomerContactEdit,
	onCustomerContactCancel,
	onCustomerContactSubmit,
	onCustomerContactDelete,
	onCustomerContextChange,
	onCustomerContextEdit,
	onCustomerContextCancel,
	onCustomerContextSubmit,
	onCustomerContextDelete,
	onCustomerContextPhotoChange,
	onRetryCustomerContacts,
	onRetryCustomerContexts,
}: CustomerEditUIProps) {
	const displayName = `${form.firstName} ${form.lastName}`.trim()
	const canSubmitCustomerContact =
		Boolean(customerContactDraft.firstName.trim()) &&
		Boolean(customerContactDraft.lastName.trim())
	const canSubmitCustomerContext = Boolean(customerContextDraft.label.trim())

	return (
		<PageShell className="pb-24 md:pb-28">
			<div>
				<Link
					to="/crm/customers"
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
				eyebrow={customerStatusLabel(form.status)}
				actions={
					form.status === 'PROSPECT' ? (
						<Button
							type="button"
							variant="outline"
							onClick={onPromoteToClient}
							disabled={isSaving}
							className="border-success/35 bg-success-soft text-success hover:bg-success-soft/80 hover:text-success"
						>
							<UserCheck />
							Promouvoir en client
						</Button>
					) : null
				}
				leading={
					<EntityAvatar tone="brand" size="lg">
						{customerInitials(customer)}
					</EntityAvatar>
				}
			/>

			<div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
				<Section
					title="Identité"
					description="Informations principales et statut CRM"
					className="lg:col-span-2"
				>
					<div className="grid gap-4 md:grid-cols-2">
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
					</div>
					<div className="flex flex-col gap-2">
						<Label htmlFor="customer-status">
							Statut CRM
							{changedKeys.includes('status') ? <Dot /> : null}
						</Label>
						<Select
							value={form.status}
							onValueChange={(status) =>
								onChange({ status: status as CustomerStatus })
							}
						>
							<SelectTrigger id="customer-status" className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value="PROSPECT">Prospect</SelectItem>
								<SelectItem value="CLIENT">Client</SelectItem>
								<SelectItem value="ARCHIVED">Archivé</SelectItem>
							</SelectContent>
						</Select>
					</div>
					<div className="flex flex-col gap-2">
						<Label htmlFor="customer-pipeline-stage">
							Étape pipeline
							{changedKeys.includes('pipelineStage') ? <Dot /> : null}
						</Label>
						<Select
							value={form.pipelineStage}
							onValueChange={(pipelineStage) =>
								onChange({
									pipelineStage: pipelineStage as CustomerPipelineStage,
								})
							}
						>
							<SelectTrigger id="customer-pipeline-stage" className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{CUSTOMER_PIPELINE_STAGE_OPTIONS.map((stage) => (
									<SelectItem key={stage} value={stage}>
										{customerPipelineStageLabel(stage)}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>
				</Section>

				<Section title="Coordonnées" description="Email et téléphone généraux">
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
					title="Contacts"
					description="Interlocuteurs liés à ce prospect ou client"
					className="lg:col-span-3"
				>
					{customerContactsError ? (
						<InlineError
							message={customerContactsError}
							onRetry={onRetryCustomerContacts}
						/>
					) : null}

					{isCustomerContactsLoading ? (
						<EmptyState label="Chargement des contacts…" />
					) : customerContacts.length === 0 ? (
						<EmptyState
							icon={
								<UserRound className="mx-auto size-6 text-muted-foreground" />
							}
							label="Aucun contact renseigné"
							description="Ajoutez au moins un interlocuteur pour qualifier la relation."
						/>
					) : (
						<div className="grid gap-3 lg:grid-cols-2">
							{customerContacts.map((customerContact) => (
								<CustomerContactCard
									key={customerContact.id}
									customerContact={customerContact}
									isDeleting={deletingCustomerContactId === customerContact.id}
									onEdit={() => onCustomerContactEdit(customerContact)}
									onDelete={() => onCustomerContactDelete(customerContact)}
								/>
							))}
						</div>
					)}

					<div className="mt-1 rounded-lg border bg-muted/20">
						<div className="flex items-center justify-between gap-4 border-b px-4 py-3">
							<div>
								<p className="font-medium">
									{editingCustomerContactId
										? 'Modifier le contact'
										: 'Ajouter un contact'}
								</p>
								<p className="text-xs text-muted-foreground">
									Nom et prénom sont requis. Le rôle aide à identifier le bon
									interlocuteur.
								</p>
							</div>
							{editingCustomerContactId ? (
								<Button
									type="button"
									variant="ghost"
									size="sm"
									onClick={onCustomerContactCancel}
								>
									Annuler
								</Button>
							) : null}
						</div>
						<div className="grid gap-4 p-4 md:grid-cols-2">
							<Field
								label="Prénom"
								name="customer-contact-first-name"
								value={customerContactDraft.firstName}
								onChange={(v) => onCustomerContactChange({ firstName: v })}
							/>
							<Field
								label="Nom"
								name="customer-contact-last-name"
								value={customerContactDraft.lastName}
								onChange={(v) => onCustomerContactChange({ lastName: v })}
							/>
							<Field
								label="Rôle"
								name="customer-contact-role"
								value={customerContactDraft.role}
								onChange={(v) => onCustomerContactChange({ role: v })}
							/>
							<Field
								label="Email"
								name="customer-contact-email"
								type="email"
								value={customerContactDraft.email}
								onChange={(v) => onCustomerContactChange({ email: v })}
							/>
							<Field
								label="Téléphone"
								name="customer-contact-phone"
								value={customerContactDraft.phone}
								onChange={(v) => onCustomerContactChange({ phone: v })}
							/>
							<div className="flex items-center justify-between gap-4 rounded-lg border bg-card px-3 py-2">
								<div>
									<Label htmlFor="customer-contact-primary">
										Contact principal
									</Label>
									<p className="text-xs text-muted-foreground">
										Affiché en priorité dans la fiche.
									</p>
								</div>
								<Switch
									id="customer-contact-primary"
									checked={customerContactDraft.isPrimary}
									onCheckedChange={(isPrimary) =>
										onCustomerContactChange({ isPrimary })
									}
								/>
							</div>
						</div>
						<div className="flex justify-end border-t p-4">
							<Button
								type="button"
								disabled={!canSubmitCustomerContact || isCustomerContactSaving}
								onClick={onCustomerContactSubmit}
							>
								<Plus />
								{editingCustomerContactId ? 'Enregistrer' : 'Ajouter'}
							</Button>
						</div>
					</div>
				</Section>

				<Section
					title="Contextes client"
					description="Adresses, établissements ou périmètres associés à ce client"
					className="lg:col-span-3"
				>
					{customerContextsError ? (
						<InlineError
							message={customerContextsError}
							onRetry={onRetryCustomerContexts}
						/>
					) : null}

					{isCustomerContextsLoading ? (
						<EmptyState label="Chargement des contextes…" />
					) : customerContexts.length === 0 ? (
						<EmptyState
							icon={
								<LayoutPanelTop className="mx-auto size-6 text-muted-foreground" />
							}
							label="Aucun contexte renseigné"
							description="Ajoutez un contexte pour ce client."
						/>
					) : (
						<div className="grid gap-3 lg:grid-cols-2">
							{customerContexts.map((customerContext) => (
								<CustomerContextCard
									key={customerContext.id}
									customerContext={customerContext}
									isDeleting={deletingCustomerContextId === customerContext.id}
									onEdit={() => onCustomerContextEdit(customerContext)}
									onDelete={() => onCustomerContextDelete(customerContext)}
								/>
							))}
						</div>
					)}

					<div className="mt-1 rounded-lg border bg-muted/20">
						<div className="flex items-center justify-between gap-4 border-b px-4 py-3">
							<div>
								<p className="font-medium">
									{editingCustomerContextId
										? 'Modifier le contexte'
										: 'Ajouter un contexte'}
								</p>
								<p className="text-xs text-muted-foreground">
									Le libellé suffit. L’adresse reste optionnelle selon le
									contexte.
								</p>
							</div>
							{editingCustomerContextId ? (
								<Button
									type="button"
									variant="ghost"
									size="sm"
									onClick={onCustomerContextCancel}
								>
									Annuler
								</Button>
							) : null}
						</div>
						<div className="grid gap-4 p-4 md:grid-cols-2">
							<Field
								label="Libellé"
								name="customer-context-label"
								value={customerContextDraft.label}
								onChange={(v) => onCustomerContextChange({ label: v })}
							/>
							<Field
								label="Ville"
								name="customer-context-city"
								value={customerContextDraft.city}
								onChange={(v) => onCustomerContextChange({ city: v })}
							/>
							<Field
								label="Adresse"
								name="customer-context-address-line"
								value={customerContextDraft.addressLine}
								onChange={(v) => onCustomerContextChange({ addressLine: v })}
							/>
							<Field
								label="Code postal"
								name="customer-context-postal-code"
								value={customerContextDraft.postalCode}
								onChange={(v) => onCustomerContextChange({ postalCode: v })}
							/>
						</div>
						<div className="grid gap-4 border-t p-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
							<div className="flex flex-col gap-2">
								<Label htmlFor="customer-context-photo">
									Photo du contexte
								</Label>
								<div className="flex flex-col gap-3 sm:flex-row sm:items-center">
									<label
										htmlFor="customer-context-photo"
										className="inline-flex h-9 cursor-pointer items-center justify-center gap-2 rounded-lg border border-primary/40 bg-card px-4 text-sm font-medium text-primary shadow-xs hover:bg-brand-soft"
									>
										<Upload className="size-4" />
										{isUploadingPhoto ? 'Téléversement…' : 'Téléverser'}
									</label>
									<input
										id="customer-context-photo"
										type="file"
										accept="image/*"
										className="sr-only"
										disabled={isUploadingPhoto}
										onChange={(event) => {
											const file = event.target.files?.[0]
											if (file) onCustomerContextPhotoChange(file)
											event.currentTarget.value = ''
										}}
									/>
									{customerContextDraft.photoKey ? (
										<span className="min-w-0 truncate font-mono text-xs text-muted-foreground">
											{customerContextDraft.photoKey}
										</span>
									) : null}
								</div>
							</div>

							{photoPreviewUrl ? (
								<img
									src={photoPreviewUrl}
									alt="Aperçu du contexte"
									className="h-24 w-36 rounded-lg border object-cover"
								/>
							) : null}
						</div>
						<div className="flex justify-end border-t p-4">
							<Button
								type="button"
								disabled={!canSubmitCustomerContext || isCustomerContextSaving}
								onClick={onCustomerContextSubmit}
							>
								<Plus />
								{editingCustomerContextId ? 'Enregistrer' : 'Ajouter'}
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

function InlineError({
	message,
	onRetry,
}: {
	message: string
	onRetry: () => void
}) {
	return (
		<div className="flex flex-col gap-3 rounded-lg border border-destructive/30 bg-destructive-soft p-4 text-destructive sm:flex-row sm:items-center sm:justify-between">
			<div className="flex items-center gap-2">
				<AlertCircle className="size-4" />
				<p className="text-sm font-medium">{message}</p>
			</div>
			<Button type="button" variant="outline" size="sm" onClick={onRetry}>
				Réessayer
			</Button>
		</div>
	)
}

function EmptyState({
	icon,
	label,
	description,
}: {
	icon?: React.ReactNode
	label: string
	description?: string
}) {
	return (
		<div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
			{icon}
			<p className={icon ? 'mt-2 font-medium text-foreground' : ''}>{label}</p>
			{description ? <p className="mt-1">{description}</p> : null}
		</div>
	)
}

interface CustomerContactCardProps {
	customerContact: CustomerContact
	isDeleting: boolean
	onEdit: () => void
	onDelete: () => void
}

function CustomerContactCard({
	customerContact,
	isDeleting,
	onEdit,
	onDelete,
}: CustomerContactCardProps) {
	const displayName =
		`${customerContact.first_name} ${customerContact.last_name}`.trim()

	return (
		<div className="flex gap-4 rounded-lg border bg-card p-4">
			<div className="flex size-12 shrink-0 items-center justify-center rounded-lg bg-brand-soft text-primary">
				<UserRound className="size-5" />
			</div>
			<div className="min-w-0 flex-1">
				<div className="flex items-start justify-between gap-3">
					<div className="min-w-0">
						<div className="flex flex-wrap items-center gap-2">
							<p className="truncate font-semibold">{displayName}</p>
							{customerContact.is_primary ? (
								<StatusBadge tone="brand">
									<Star className="size-3" />
									Principal
								</StatusBadge>
							) : null}
						</div>
						{customerContact.role ? (
							<p className="mt-1 text-sm text-muted-foreground">
								{customerContact.role}
							</p>
						) : null}
						<div className="mt-2 flex flex-col gap-1 text-xs text-muted-foreground">
							<span className="flex min-w-0 items-center gap-1">
								<Mail className="size-3" />
								<span className="truncate">
									{customerContact.email || 'Email non renseigné'}
								</span>
							</span>
							<span className="flex items-center gap-1">
								<Phone className="size-3" />
								{customerContact.phone || 'Téléphone non renseigné'}
							</span>
						</div>
					</div>
					<RowActions
						isDeleting={isDeleting}
						onEdit={onEdit}
						onDelete={onDelete}
					/>
				</div>
			</div>
		</div>
	)
}

interface CustomerContextCardProps {
	customerContext: CustomerContext
	isDeleting: boolean
	onEdit: () => void
	onDelete: () => void
}

function CustomerContextCard({
	customerContext,
	isDeleting,
	onEdit,
	onDelete,
}: CustomerContextCardProps) {
	const addressParts = [
		customerContext.address_line,
		[customerContext.postal_code, customerContext.city]
			.filter(Boolean)
			.join(' '),
	].filter(Boolean)

	return (
		<div className="flex gap-4 rounded-lg border bg-card p-4">
			<div className="flex size-16 shrink-0 items-center justify-center overflow-hidden rounded-lg border bg-muted">
				{customerContext.photo_key ? (
					<Image className="size-6 text-muted-foreground" />
				) : (
					<LayoutPanelTop className="size-6 text-muted-foreground" />
				)}
			</div>
			<div className="min-w-0 flex-1">
				<div className="flex items-start justify-between gap-3">
					<div className="min-w-0">
						<p className="truncate font-semibold">{customerContext.label}</p>
						{addressParts.length > 0 ? (
							<div className="mt-1 text-sm text-muted-foreground">
								{addressParts.map((part) => (
									<p key={part}>{part}</p>
								))}
							</div>
						) : (
							<p className="mt-1 text-sm text-muted-foreground">
								Aucune adresse renseignée
							</p>
						)}
					</div>
					<RowActions
						isDeleting={isDeleting}
						onEdit={onEdit}
						onDelete={onDelete}
					/>
				</div>
				{customerContext.photo_key ? (
					<p className="mt-2 truncate font-mono text-xs text-muted-foreground">
						photo: {customerContext.photo_key}
					</p>
				) : null}
			</div>
		</div>
	)
}

function RowActions({
	isDeleting,
	onEdit,
	onDelete,
}: {
	isDeleting: boolean
	onEdit: () => void
	onDelete: () => void
}) {
	return (
		<div className="flex shrink-0 gap-1">
			<Button type="button" variant="ghost" size="icon-sm" onClick={onEdit}>
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
	)
}
