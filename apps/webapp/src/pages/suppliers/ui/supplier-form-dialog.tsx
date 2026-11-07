import { Plus, Trash2 } from 'lucide-react'
import { type FormEvent, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
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
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { FormSection } from '#/pages/customers/ui/form-section'
import { Money } from '#/pages/customers/ui/money'
import { TextArea } from '#/pages/customers/ui/text-area'
import {
	emptyAddress,
	emptyContact,
	supplierFormErrors,
} from '#/pages/suppliers/lib/supplier-form'
import {
	ADDRESS_KIND_LABELS,
	ADDRESS_KINDS,
	type AddressKind,
	type SupplierAddress,
	type SupplierContact,
	type SupplierFormValues,
} from '#/pages/suppliers/types'
import type { ExpenseCategory } from '#/pages/treasury/types'
import { CategoryChip } from '#/pages/treasury/ui/category-chip'

const NO_CATEGORY = '__none'

export interface SupplierFormDialogProps {
	open: boolean
	editingName: string | null
	initialValues: SupplierFormValues
	categories: readonly ExpenseCategory[]
	archived: boolean | null
	toPay: number
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: SupplierFormValues) => void
	onToggleArchive: () => void
}

export function SupplierFormDialog({
	open,
	onOpenChange,
	...form
}: SupplierFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<SupplierForm onClose={() => onOpenChange(false)} {...form} />
			</DialogContent>
		</Dialog>
	)
}

function SupplierForm({
	editingName,
	initialValues,
	categories,
	archived,
	toPay,
	isPending,
	error,
	onClose,
	onSubmit,
	onToggleArchive,
}: Omit<SupplierFormDialogProps, 'open' | 'onOpenChange'> & {
	onClose: () => void
}) {
	const baseId = useId()
	const [values, setValues] = useState<SupplierFormValues>(initialValues)
	const [attempted, setAttempted] = useState(false)
	const id = (field: string) => `${baseId}-${field}`
	const errors = supplierFormErrors(values)

	function change(patch: Partial<SupplierFormValues>) {
		setValues((current) => ({ ...current, ...patch }))
	}

	function changeContact(contactId: string, patch: Partial<SupplierContact>) {
		setValues((current) => ({
			...current,
			contacts: current.contacts.map((contact) =>
				contact.id === contactId ? { ...contact, ...patch } : contact,
			),
		}))
	}

	function changeAddress(addressId: string, patch: Partial<SupplierAddress>) {
		setValues((current) => ({
			...current,
			addresses: current.addresses.map((address) =>
				address.id === addressId ? { ...address, ...patch } : address,
			),
		}))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		if (errors.length > 0) {
			setAttempted(true)
			return
		}
		onSubmit(values)
	}

	return (
		<form onSubmit={submit} noValidate className="flex min-h-0 flex-1 flex-col">
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>
					{editingName ? `Modifier ${editingName}` : 'Nouveau fournisseur'}
				</DialogTitle>
				<DialogDescription>
					{editingName
						? archived
							? 'Ce fournisseur est archivé : il n’est plus proposé à la saisie des dépenses.'
							: 'Mets à jour sa fiche. La catégorie par défaut est proposée à chaque nouvelle dépense.'
						: 'Renseigne l’essentiel, tu pourras compléter la fiche plus tard.'}
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-6 overflow-y-auto border-t px-6 py-5">
				<FormSection label="Identité">
					<FormField
						id={id('name')}
						label="Nom"
						required
						className="sm:col-span-2"
					>
						<Input
							id={id('name')}
							value={values.name}
							onChange={(event) => change({ name: event.target.value })}
							autoComplete="off"
						/>
					</FormField>
					<FormField
						id={id('registration')}
						label="SIRET"
						hint="14 chiffres, espaces acceptés."
					>
						<Input
							id={id('registration')}
							inputMode="numeric"
							value={values.registrationNumber}
							onChange={(event) =>
								change({ registrationNumber: event.target.value })
							}
							autoComplete="off"
						/>
					</FormField>
					<FormField id={id('vat')} label="N° de TVA">
						<Input
							id={id('vat')}
							value={values.vatNumber}
							onChange={(event) => change({ vatNumber: event.target.value })}
							autoComplete="off"
							placeholder="FR12345678901"
						/>
					</FormField>
					<FormField id={id('iban')} label="IBAN" className="sm:col-span-2">
						<Input
							id={id('iban')}
							value={values.iban}
							onChange={(event) => change({ iban: event.target.value })}
							autoComplete="off"
						/>
					</FormField>
				</FormSection>
				<FormSection label="Achats">
					<FormField id={id('category')} label="Catégorie par défaut">
						<Select
							value={values.defaultCategoryId ?? NO_CATEGORY}
							onValueChange={(value) =>
								change({
									defaultCategoryId: value === NO_CATEGORY ? null : value,
								})
							}
						>
							<SelectTrigger id={id('category')} className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value={NO_CATEGORY}>Aucune</SelectItem>
								{categories.map((category) => (
									<SelectItem key={category.id} value={category.id}>
										<CategoryChip category={category} />
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</FormField>
					<FormField
						id={id('terms')}
						label="Délai de paiement (jours)"
						hint="Laisse vide si le fournisseur n’a pas de délai habituel."
					>
						<Input
							id={id('terms')}
							inputMode="numeric"
							value={values.paymentTermsDays}
							onChange={(event) =>
								change({ paymentTermsDays: event.target.value })
							}
							autoComplete="off"
						/>
					</FormField>
				</FormSection>
				<div className="grid gap-3">
					{values.contacts.map((contact, index) => (
						<ContactFields
							key={contact.id}
							contact={contact}
							index={index}
							baseId={id(`contact-${contact.id}`)}
							onChange={(patch) => changeContact(contact.id, patch)}
							onRemove={
								values.contacts.length > 1
									? () =>
											change({
												contacts: values.contacts.filter(
													(item) => item.id !== contact.id,
												),
											})
									: undefined
							}
						/>
					))}
					<div>
						<Button
							type="button"
							variant="outline"
							size="sm"
							onClick={() =>
								change({
									contacts: [
										...values.contacts,
										emptyContact(crypto.randomUUID()),
									],
								})
							}
						>
							<Plus aria-hidden /> Ajouter un contact
						</Button>
					</div>
				</div>
				<div className="grid gap-3">
					{values.addresses.map((address, index) => (
						<AddressFields
							key={address.id}
							address={address}
							index={index}
							baseId={id(`address-${address.id}`)}
							onChange={(patch) => changeAddress(address.id, patch)}
							onRemove={
								values.addresses.length > 1
									? () =>
											change({
												addresses: values.addresses.filter(
													(item) => item.id !== address.id,
												),
											})
									: undefined
							}
						/>
					))}
					<div>
						<Button
							type="button"
							variant="outline"
							size="sm"
							onClick={() =>
								change({
									addresses: [
										...values.addresses,
										{
											...emptyAddress(crypto.randomUUID()),
											label: 'Adresse',
											kind: 'other',
										},
									],
								})
							}
						>
							<Plus aria-hidden /> Ajouter une adresse
						</Button>
					</div>
				</div>
				<FormSection label="Notes">
					<FormField
						id={id('notes')}
						label="Notes internes"
						className="sm:col-span-2"
					>
						<TextArea
							id={id('notes')}
							rows={3}
							value={values.notes}
							onChange={(event) => change({ notes: event.target.value })}
						/>
					</FormField>
				</FormSection>
				{editingName && toPay > 0 && (
					<p className="text-[13px] text-muted-foreground">
						Il reste <Money cents={toPay} /> à payer à ce fournisseur :
						l’archivage ne supprime pas ces dépenses.
					</p>
				)}
			</div>
			<div className="grid gap-3 border-t px-6 py-4">
				{attempted && errors.length > 0 && (
					<ul
						role="alert"
						className="list-disc rounded-md bg-state-alert/8 py-2 pr-3 pl-7 text-[13px] text-state-alert"
					>
						{errors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
				{error ? (
					<p
						role="alert"
						className="rounded-md bg-state-alert/8 px-3 py-2 text-[13px] text-state-alert"
					>
						{error}
					</p>
				) : null}
				<DialogFooter className="sm:justify-between">
					{editingName ? (
						<Button type="button" variant="outline" onClick={onToggleArchive}>
							{archived ? 'Réactiver' : 'Archiver'}
						</Button>
					) : (
						<span />
					)}
					<div className="flex flex-col-reverse gap-2 sm:flex-row">
						<Button type="button" variant="outline" onClick={onClose}>
							Annuler
						</Button>
						<Button type="submit" disabled={isPending}>
							{editingName ? 'Enregistrer' : 'Créer le fournisseur'}
						</Button>
					</div>
				</DialogFooter>
			</div>
		</form>
	)
}

function ContactFields({
	contact,
	index,
	baseId,
	onChange,
	onRemove,
}: {
	contact: SupplierContact
	index: number
	baseId: string
	onChange: (patch: Partial<SupplierContact>) => void
	onRemove?: () => void
}) {
	return (
		<div className="grid gap-3">
			<div className="flex items-center justify-between gap-2">
				<h3 className="font-medium text-sm">
					{index === 0 ? 'Contact principal' : `Contact ${index + 1}`}
				</h3>
				{onRemove && (
					<Button
						type="button"
						variant="ghost"
						size="sm"
						onClick={onRemove}
						aria-label={`Retirer le contact ${index + 1}`}
					>
						<Trash2 aria-hidden /> Retirer
					</Button>
				)}
			</div>
			<div className="grid gap-3 sm:grid-cols-2">
				<FormField id={`${baseId}-first`} label="Prénom">
					<Input
						id={`${baseId}-first`}
						value={contact.firstName}
						onChange={(event) => onChange({ firstName: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField id={`${baseId}-last`} label="Nom">
					<Input
						id={`${baseId}-last`}
						value={contact.lastName}
						onChange={(event) => onChange({ lastName: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField
					id={`${baseId}-role`}
					label="Fonction"
					className="sm:col-span-2"
				>
					<Input
						id={`${baseId}-role`}
						value={contact.role}
						onChange={(event) => onChange({ role: event.target.value })}
						autoComplete="off"
						placeholder="Commercial, comptabilité…"
					/>
				</FormField>
				<FormField id={`${baseId}-phone`} label="Téléphone">
					<Input
						id={`${baseId}-phone`}
						type="tel"
						value={contact.phone}
						onChange={(event) => onChange({ phone: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField id={`${baseId}-mobile`} label="Mobile">
					<Input
						id={`${baseId}-mobile`}
						type="tel"
						value={contact.mobile}
						onChange={(event) => onChange({ mobile: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField
					id={`${baseId}-email`}
					label="E-mail"
					className="sm:col-span-2"
				>
					<Input
						id={`${baseId}-email`}
						type="email"
						value={contact.email}
						onChange={(event) => onChange({ email: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
			</div>
		</div>
	)
}

function AddressFields({
	address,
	index,
	baseId,
	onChange,
	onRemove,
}: {
	address: SupplierAddress
	index: number
	baseId: string
	onChange: (patch: Partial<SupplierAddress>) => void
	onRemove?: () => void
}) {
	return (
		<div className="grid gap-3">
			<div className="flex items-center justify-between gap-2">
				<h3 className="font-medium text-sm">
					{index === 0 ? 'Adresse principale' : `Adresse ${index + 1}`}
				</h3>
				{onRemove && (
					<Button
						type="button"
						variant="ghost"
						size="sm"
						onClick={onRemove}
						aria-label={`Retirer l’adresse ${index + 1}`}
					>
						<Trash2 aria-hidden /> Retirer
					</Button>
				)}
			</div>
			<div className="grid gap-3 sm:grid-cols-2">
				<FormField id={`${baseId}-label`} label="Libellé">
					<Input
						id={`${baseId}-label`}
						value={address.label}
						onChange={(event) => onChange({ label: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField id={`${baseId}-kind`} label="Type">
					<Select
						value={address.kind}
						onValueChange={(value) => onChange({ kind: value as AddressKind })}
					>
						<SelectTrigger id={`${baseId}-kind`} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{ADDRESS_KINDS.map((kind) => (
								<SelectItem key={kind} value={kind}>
									{ADDRESS_KIND_LABELS[kind]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<FormField
					id={`${baseId}-street`}
					label="Rue"
					className="sm:col-span-2"
				>
					<Input
						id={`${baseId}-street`}
						value={address.street}
						onChange={(event) => onChange({ street: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField
					id={`${baseId}-complement`}
					label="Complément"
					className="sm:col-span-2"
				>
					<Input
						id={`${baseId}-complement`}
						value={address.complement}
						onChange={(event) => onChange({ complement: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField id={`${baseId}-postal`} label="Code postal">
					<Input
						id={`${baseId}-postal`}
						inputMode="numeric"
						value={address.postalCode}
						onChange={(event) => onChange({ postalCode: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
				<FormField id={`${baseId}-city`} label="Ville">
					<Input
						id={`${baseId}-city`}
						value={address.city}
						onChange={(event) => onChange({ city: event.target.value })}
						autoComplete="off"
					/>
				</FormField>
			</div>
		</div>
	)
}
