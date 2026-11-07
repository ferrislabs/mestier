import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
import type {
	AccountExtras,
	AccountKind,
	LeadSource,
} from '#/pages/customers/feature/mocks/account-extras'

export type ContactForm = {
	firstName: string
	lastName: string
	role: string
	email: string
	mobile: string
	phone: string
}

export type AddressForm = {
	street: string
	complement: string
	postalCode: string
	city: string
	country: string
}

export type AccountForm = {
	kind: AccountKind
	status: 'prospect' | 'client'
	pipelineStage: CustomerPipelineStage
	name: string
	registrationNumber: string
	vatNumber: string
	website: string
	email: string
	phone: string
	ownerId: string | null
	source: LeadSource | null
	paymentTerms: string
	stripeCustomerId: string
	notes: string
	contact: ContactForm
	address: AddressForm
}

export type FieldKey =
	| 'name'
	| 'registrationNumber'
	| 'email'
	| 'phone'
	| 'paymentTerms'
	| 'stripeCustomerId'
	| 'contactName'
	| 'contactEmail'
	| 'contactMobile'
	| 'contactPhone'
	| 'street'
	| 'city'
	| 'postalCode'

export type FieldErrors = Partial<Record<FieldKey, string>>

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
const PHONE_PATTERN = /^\+?[0-9][0-9 .-]{5,18}$/
const POSTAL_CODE_PATTERN = /^\d{5}$/
const SIRET_PATTERN = /^(\d{14}|\d{9})$/
const STRIPE_CUSTOMER_PATTERN = /^cus_[A-Za-z0-9]+$/

export const EMPTY_CONTACT: ContactForm = {
	firstName: '',
	lastName: '',
	role: '',
	email: '',
	mobile: '',
	phone: '',
}

export const EMPTY_ADDRESS: AddressForm = {
	street: '',
	complement: '',
	postalCode: '',
	city: '',
	country: 'France',
}

export function emptyAccountForm(ownerId: string | null): AccountForm {
	return {
		kind: 'professionnel',
		status: 'prospect',
		pipelineStage: 'NEW',
		name: '',
		registrationNumber: '',
		vatNumber: '',
		website: '',
		email: '',
		phone: '',
		ownerId,
		source: null,
		paymentTerms: '',
		stripeCustomerId: '',
		notes: '',
		contact: { ...EMPTY_CONTACT },
		address: { ...EMPTY_ADDRESS },
	}
}

export function accountFormOf(
	customer: Customer,
	extras: AccountExtras,
	kind: AccountKind,
): AccountForm {
	return {
		...emptyAccountForm(extras.ownerId),
		kind,
		status: customer.status === 'PROSPECT' ? 'prospect' : 'client',
		pipelineStage: customer.pipeline_stage,
		name: customer.name,
		registrationNumber: customer.registration_number ?? '',
		vatNumber: extras.vatNumber,
		website: extras.website,
		email: customer.email ?? '',
		phone: customer.phone ?? '',
		source: extras.source,
		paymentTerms:
			extras.paymentTermsDays === null ? '' : String(extras.paymentTermsDays),
		stripeCustomerId: extras.stripeCustomerId ?? '',
		notes: extras.notes,
	}
}

export function parsePaymentTerms(input: string): number | null {
	const trimmed = input.trim()
	if (trimmed === '') return null
	return /^\d+$/.test(trimmed) ? Number(trimmed) : Number.NaN
}

export function isContactEmpty(contact: ContactForm): boolean {
	return (
		contact.firstName.trim() === '' &&
		contact.lastName.trim() === '' &&
		contact.email.trim() === '' &&
		contact.mobile.trim() === '' &&
		contact.phone.trim() === ''
	)
}

export function isAddressEmpty(address: AddressForm): boolean {
	return (
		address.street.trim() === '' &&
		address.postalCode.trim() === '' &&
		address.city.trim() === ''
	)
}

export function validateContactForm(contact: ContactForm): FieldErrors {
	const errors: FieldErrors = {}
	if (contact.firstName.trim() === '' || contact.lastName.trim() === '')
		errors.contactName = 'Renseigne le prénom et le nom du contact.'
	if (contact.email.trim() !== '' && !EMAIL_PATTERN.test(contact.email.trim()))
		errors.contactEmail = 'L’adresse e-mail du contact n’est pas valide.'
	if (
		contact.mobile.trim() !== '' &&
		!PHONE_PATTERN.test(contact.mobile.trim())
	)
		errors.contactMobile = 'Le numéro de mobile n’est pas valide.'
	if (contact.phone.trim() !== '' && !PHONE_PATTERN.test(contact.phone.trim()))
		errors.contactPhone = 'Le numéro de téléphone n’est pas valide.'
	return errors
}

export function validateAddressForm(address: AddressForm): FieldErrors {
	const errors: FieldErrors = {}
	if (address.street.trim() === '') errors.street = 'La rue est obligatoire.'
	if (address.city.trim() === '') errors.city = 'La ville est obligatoire.'
	if (!POSTAL_CODE_PATTERN.test(address.postalCode.trim()))
		errors.postalCode = 'Le code postal doit comporter 5 chiffres.'
	return errors
}

export function validateAccountForm(
	form: AccountForm,
	mode: 'create' | 'edit',
): FieldErrors {
	const errors: FieldErrors = {}
	if (form.name.trim() === '') errors.name = 'Le nom est obligatoire.'
	if (form.email.trim() !== '' && !EMAIL_PATTERN.test(form.email.trim()))
		errors.email = 'L’adresse e-mail n’est pas valide.'
	if (form.phone.trim() !== '' && !PHONE_PATTERN.test(form.phone.trim()))
		errors.phone = 'Le numéro de téléphone n’est pas valide.'
	const registration = form.registrationNumber.replace(/\s/g, '')
	if (
		form.kind === 'professionnel' &&
		registration !== '' &&
		!SIRET_PATTERN.test(registration)
	)
		errors.registrationNumber = 'Le SIRET doit comporter 14 chiffres.'
	const terms = parsePaymentTerms(form.paymentTerms)
	if (terms !== null && (!Number.isInteger(terms) || terms < 0 || terms > 60))
		errors.paymentTerms =
			'Le délai de paiement doit être compris entre 0 et 60 jours.'
	const stripe = form.stripeCustomerId.trim()
	if (stripe !== '' && !STRIPE_CUSTOMER_PATTERN.test(stripe))
		errors.stripeCustomerId =
			'L’identifiant Stripe doit commencer par « cus_ » et ne contenir ni espace ni caractère spécial.'
	if (mode === 'create') {
		if (!isContactEmpty(form.contact))
			Object.assign(errors, validateContactForm(form.contact))
		if (!isAddressEmpty(form.address))
			Object.assign(errors, validateAddressForm(form.address))
	}
	return errors
}

export type FormExtras = Pick<
	AccountExtras,
	| 'kind'
	| 'vatNumber'
	| 'website'
	| 'ownerId'
	| 'source'
	| 'paymentTermsDays'
	| 'stripeCustomerId'
	| 'notes'
>

export function registrationNumberOf(form: AccountForm): string | null {
	if (form.kind !== 'professionnel') return null
	return form.registrationNumber.replace(/\s/g, '') || null
}

export function extrasOfForm(form: AccountForm): FormExtras {
	const stripe = form.stripeCustomerId.trim()
	return {
		kind: form.kind,
		vatNumber: form.vatNumber.trim(),
		website: form.website.trim(),
		ownerId: form.ownerId,
		source: form.source,
		paymentTermsDays: parsePaymentTerms(form.paymentTerms),
		stripeCustomerId: stripe === '' ? null : stripe,
		notes: form.notes.trim(),
	}
}
