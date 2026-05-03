import { Link } from '@tanstack/react-router'
import { ArrowLeft } from 'lucide-react'
import { FloatingActionBar } from '#/components/floating-action-bar'
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
} from '#/components/ui/surface'
import {
	CATEGORY_LABELS,
	type Customer,
	type CustomerCategory,
} from '#/pages/customers/types'

const CATEGORY_TONE: Record<CustomerCategory, 'brand' | 'success' | 'warning'> =
	{
		artisan: 'warning',
		sme: 'brand',
		individual: 'success',
	}

const CATEGORIES: CustomerCategory[] = ['artisan', 'sme', 'individual']

interface CustomerEditUIProps {
	customer: Customer
	form: Customer
	isDirty: boolean
	changedKeys: (keyof Customer)[]
	isSaving: boolean
	onChange: (patch: Partial<Customer>) => void
	onAddressChange: (patch: Partial<Customer['address']>) => void
	onReset: () => void
	onSave: () => void
}

export function CustomerEditUI({
	customer,
	form,
	isDirty,
	changedKeys,
	isSaving,
	onChange,
	onAddressChange,
	onReset,
	onSave,
}: CustomerEditUIProps) {
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
				title={form.name || 'Nouveau client'}
				description={
					<span className="font-mono text-xs">id: {customer.id}</span>
				}
				className="items-center sm:items-center sm:justify-start"
				eyebrow={CATEGORY_LABELS[form.category]}
				leading={
					<EntityAvatar tone={CATEGORY_TONE[form.category]} size="lg">
						{form.name[0]?.toUpperCase()}
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
						label="Nom"
						name="name"
						value={form.name}
						onChange={(v) => onChange({ name: v })}
						changed={changedKeys.includes('name')}
					/>
					<Field
						label="Contact"
						name="contact_name"
						value={form.contact_name}
						onChange={(v) => onChange({ contact_name: v })}
						changed={changedKeys.includes('contact_name')}
					/>
					<div className="flex flex-col gap-2">
						<Label htmlFor="category">
							Catégorie
							{changedKeys.includes('category') ? <Dot /> : null}
						</Label>
						<Select
							value={form.category}
							onValueChange={(v) =>
								onChange({ category: v as CustomerCategory })
							}
						>
							<SelectTrigger id="category">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{CATEGORIES.map((c) => (
									<SelectItem key={c} value={c}>
										{CATEGORY_LABELS[c]}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>
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
					title="Adresse"
					description="Localisation du client"
					className="lg:col-span-3"
				>
					<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
						<Field
							label="Rue"
							name="street"
							value={form.address.street}
							onChange={(v) => onAddressChange({ street: v })}
							changed={changedKeys.includes('address')}
						/>
						<Field
							label="Ville"
							name="city"
							value={form.address.city}
							onChange={(v) => onAddressChange({ city: v })}
							changed={changedKeys.includes('address')}
						/>
						<Field
							label="Code postal"
							name="zip"
							value={form.address.zip}
							onChange={(v) => onAddressChange({ zip: v })}
							changed={changedKeys.includes('address')}
						/>
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
