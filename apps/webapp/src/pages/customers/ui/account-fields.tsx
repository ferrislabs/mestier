import { useId } from 'react'
import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import type { AccountForm } from '#/pages/customers/lib/account-form'
import type { Person } from '#/pages/customers/lib/people'
import {
	ACCOUNT_KIND_LABELS,
	type AccountKind,
	LEAD_SOURCE_LABELS,
	LEAD_SOURCES,
} from '#/pages/customers/types'
import { TextArea } from '#/pages/customers/ui/text-area'
import type { BindField } from '#/pages/customers/ui/use-form-binding'

const NONE = '__none'

const KINDS: readonly AccountKind[] = ['professionnel', 'particulier']

const STATUSES: readonly { value: AccountForm['status']; label: string }[] = [
	{ value: 'prospect', label: 'Prospect' },
	{ value: 'client', label: 'Client' },
]

function ChoiceField<T extends string>({
	label,
	value,
	options,
	onChange,
}: {
	label: string
	value: T
	options: readonly { value: T; label: string }[]
	onChange: (next: T) => void
}) {
	const labelId = useId()
	return (
		<div className="grid content-start gap-1.5">
			<span id={labelId} className="font-medium text-sm leading-none">
				{label}
			</span>
			<ToggleGroup
				type="single"
				variant="outline"
				value={value}
				onValueChange={(next) => {
					const option = options.find((item) => item.value === next)
					if (option) onChange(option.value)
				}}
				aria-labelledby={labelId}
				className="w-full"
			>
				{options.map((option) => (
					<ToggleGroupItem
						key={option.value}
						value={option.value}
						className="flex-1"
					>
						{option.label}
					</ToggleGroupItem>
				))}
			</ToggleGroup>
		</div>
	)
}

export function AccountFields({
	form,
	onChange,
	bind,
	mode,
	defaultTerms,
	people,
}: {
	form: AccountForm
	onChange: (patch: Partial<AccountForm>) => void
	bind: BindField
	mode: 'create' | 'edit'
	defaultTerms: number
	people: readonly Person[]
}) {
	const name = bind('name')
	const registrationNumber = bind('registrationNumber')
	const email = bind('email')
	const phone = bind('phone')
	const terms = bind('paymentTerms')
	const stripe = bind('stripeCustomerId')
	const baseId = name.id
	const professional = form.kind === 'professionnel'
	return (
		<>
			<ChoiceField
				label="Type de client"
				value={form.kind}
				options={KINDS.map((kind) => ({
					value: kind,
					label: ACCOUNT_KIND_LABELS[kind],
				}))}
				onChange={(kind) => onChange({ kind })}
			/>
			{mode === 'create' ? (
				<ChoiceField
					label="Statut"
					value={form.status}
					options={STATUSES}
					onChange={(status) => onChange({ status })}
				/>
			) : (
				<div aria-hidden className="max-sm:hidden" />
			)}
			<FormField
				id={name.id}
				label={professional ? 'Raison sociale' : 'Nom du client'}
				required
				error={name.error}
				className="sm:col-span-2"
			>
				<Input
					{...name.input}
					value={form.name}
					onChange={(event) => onChange({ name: event.target.value })}
					autoComplete="off"
					placeholder={professional ? 'Menuiserie Duval' : 'M. et Mme Durand'}
				/>
			</FormField>
			{professional && (
				<FormField
					id={registrationNumber.id}
					label="SIRET"
					error={registrationNumber.error}
					hint="14 chiffres, espaces acceptés."
				>
					<Input
						{...registrationNumber.input}
						inputMode="numeric"
						value={form.registrationNumber}
						onChange={(event) =>
							onChange({ registrationNumber: event.target.value })
						}
						autoComplete="off"
						placeholder="823 456 789 00019"
					/>
				</FormField>
			)}
			<FormField
				id={`${baseId}-vat`}
				label="N° de TVA"
				className={professional ? undefined : 'sm:col-span-2'}
			>
				<Input
					id={`${baseId}-vat`}
					value={form.vatNumber}
					onChange={(event) => onChange({ vatNumber: event.target.value })}
					autoComplete="off"
					placeholder="FR12823456789"
				/>
			</FormField>
			{mode === 'edit' && (
				<>
					<FormField id={email.id} label="E-mail" error={email.error}>
						<Input
							{...email.input}
							type="email"
							value={form.email}
							onChange={(event) => onChange({ email: event.target.value })}
							autoComplete="off"
							placeholder="contact@entreprise.fr"
						/>
					</FormField>
					<FormField id={phone.id} label="Téléphone" error={phone.error}>
						<Input
							{...phone.input}
							type="tel"
							value={form.phone}
							onChange={(event) => onChange({ phone: event.target.value })}
							autoComplete="off"
							placeholder="02 40 12 34 56"
						/>
					</FormField>
					<FormField
						id={`${baseId}-website`}
						label="Site web"
						className="sm:col-span-2"
					>
						<Input
							id={`${baseId}-website`}
							type="url"
							value={form.website}
							onChange={(event) => onChange({ website: event.target.value })}
							autoComplete="off"
							placeholder="https://"
						/>
					</FormField>
				</>
			)}
			<FormField id={`${baseId}-owner`} label="Responsable">
				<Select
					value={form.ownerId ?? NONE}
					onValueChange={(value) =>
						onChange({ ownerId: value === NONE ? null : value })
					}
				>
					<SelectTrigger id={`${baseId}-owner`} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={NONE}>Aucun</SelectItem>
						{people.map((person) => (
							<SelectItem key={person.id} value={person.id}>
								{person.name}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</FormField>
			<FormField id={`${baseId}-source`} label="Source">
				<Select
					value={form.source ?? NONE}
					onValueChange={(value) =>
						onChange({
							source: LEAD_SOURCES.find((source) => source === value) ?? null,
						})
					}
				>
					<SelectTrigger id={`${baseId}-source`} className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={NONE}>Non renseignée</SelectItem>
						{LEAD_SOURCES.map((source) => (
							<SelectItem key={source} value={source}>
								{LEAD_SOURCE_LABELS[source]}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</FormField>
			<FormField
				id={terms.id}
				label="Délai de paiement (jours)"
				error={terms.error}
				hint={`Laisse vide pour appliquer le délai des réglages (${defaultTerms} jours).`}
				className="sm:col-span-2"
			>
				<Input
					{...terms.input}
					inputMode="numeric"
					value={form.paymentTerms}
					onChange={(event) => onChange({ paymentTerms: event.target.value })}
					autoComplete="off"
					placeholder={String(defaultTerms)}
				/>
			</FormField>
			{mode === 'edit' && (
				<>
					<FormField
						id={stripe.id}
						label="Identifiant client Stripe"
						error={stripe.error}
						hint="Sert à rattacher automatiquement les paiements reçus par webhook. Il commence par cus_."
						className="sm:col-span-2"
					>
						<Input
							{...stripe.input}
							value={form.stripeCustomerId}
							onChange={(event) =>
								onChange({ stripeCustomerId: event.target.value })
							}
							autoComplete="off"
							spellCheck={false}
							placeholder="cus_…"
							className="font-mono"
						/>
					</FormField>
					<FormField
						id={`${baseId}-notes`}
						label="Notes"
						className="sm:col-span-2"
					>
						<TextArea
							id={`${baseId}-notes`}
							rows={4}
							value={form.notes}
							onChange={(event) => onChange({ notes: event.target.value })}
						/>
					</FormField>
				</>
			)}
		</>
	)
}
