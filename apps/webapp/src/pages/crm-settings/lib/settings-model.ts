import type {
	Company,
	SequenceKind,
	Settings,
	StripeEventType,
	VatRate,
} from '#/pages/crm-settings/types'
import { centsToInput, parseAmount } from '#/pages/customers/lib/format'

export const SEQUENCE_LABELS: Record<SequenceKind, string> = {
	quote: 'Devis',
	invoice: 'Factures',
	creditNote: 'Avoirs',
}

export function formatNumber(
	prefix: string,
	year: number,
	sequence: number,
	padding: number,
): string {
	return `${prefix}-${year}-${String(sequence).padStart(padding, '0')}`
}

export function validateCompany(company: Company): string[] {
	const errors: string[] = []
	if (company.name.trim() === '')
		errors.push('La raison sociale est obligatoire.')
	if (!/^\d{14}$/.test(company.legalId.replace(/\s/g, '')))
		errors.push('Le SIRET doit comporter 14 chiffres.')
	if (!/^FR\d{2}\d{9}$/.test(company.vatNumber.replace(/\s/g, '')))
		errors.push('Le numéro de TVA doit être au format FR + 11 chiffres.')
	if (company.capital.trim() !== '' && parseAmount(company.capital) === null)
		errors.push('Le capital social doit être un montant en euros.')
	if (
		company.address.street.trim() === '' ||
		company.address.city.trim() === ''
	)
		errors.push('L’adresse du siège est obligatoire.')
	return errors
}

export function cleanCompany(company: Company): Company {
	return {
		...company,
		legalId: company.legalId.replace(/\s/g, ''),
		vatNumber: company.vatNumber.replace(/\s/g, ''),
	}
}

export function validateSettings(
	patch: Partial<
		Pick<
			Settings,
			| 'paymentTermsDays'
			| 'quoteValidityDays'
			| 'numberPadding'
			| 'recoveryFee'
			| 'latePenalty'
		>
	>,
): string[] {
	const errors: string[] = []
	if (
		patch.paymentTermsDays !== undefined &&
		(!Number.isInteger(patch.paymentTermsDays) ||
			patch.paymentTermsDays < 0 ||
			patch.paymentTermsDays > 60)
	)
		errors.push('Le délai de paiement doit être compris entre 0 et 60 jours.')
	if (
		patch.quoteValidityDays !== undefined &&
		(!Number.isInteger(patch.quoteValidityDays) || patch.quoteValidityDays < 1)
	)
		errors.push('La validité d’un devis doit être d’au moins 1 jour.')
	if (
		patch.numberPadding !== undefined &&
		(!Number.isInteger(patch.numberPadding) ||
			patch.numberPadding < 1 ||
			patch.numberPadding > 6)
	)
		errors.push(
			'Le nombre de chiffres du compteur doit être compris entre 1 et 6.',
		)
	if (
		patch.recoveryFee !== undefined &&
		(!Number.isInteger(patch.recoveryFee) || patch.recoveryFee < 0)
	)
		errors.push('L’indemnité de recouvrement doit être positive.')
	if (patch.latePenalty !== undefined && patch.latePenalty.trim() === '')
		errors.push('Les pénalités de retard sont obligatoires.')
	return errors
}

export type CompanyField =
	| 'name'
	| 'capital'
	| 'legalId'
	| 'vatNumber'
	| 'street'
	| 'city'

export type CompanyErrors = {
	fields: Partial<Record<CompanyField, string>>
	other: string[]
}

function companyFieldOf(
	message: string,
	company: Company,
): CompanyField | null {
	if (message.includes('raison sociale')) return 'name'
	if (message.includes('capital')) return 'capital'
	if (message.includes('SIRET')) return 'legalId'
	if (message.includes('TVA')) return 'vatNumber'
	if (message.includes('adresse'))
		return company.address.street.trim() === '' ? 'street' : 'city'
	return null
}

export function companyErrors(company: Company): CompanyErrors {
	const fields: Partial<Record<CompanyField, string>> = {}
	const other: string[] = []
	for (const message of validateCompany(company)) {
		const field = companyFieldOf(message, company)
		if (field === null) other.push(message)
		else if (fields[field] === undefined) fields[field] = message
	}
	return { fields, other }
}

export type BillingForm = {
	paymentTermsDays: string
	quoteValidityDays: string
	defaultVatRate: VatRate
	latePenalty: string
	recoveryFee: string
	invoiceFooter: string
	numberPadding: string
}

export type BillingField =
	| 'paymentTermsDays'
	| 'quoteValidityDays'
	| 'latePenalty'
	| 'recoveryFee'
	| 'numberPadding'

export type BillingPatch = Pick<
	Settings,
	| 'paymentTermsDays'
	| 'quoteValidityDays'
	| 'defaultVatRate'
	| 'latePenalty'
	| 'recoveryFee'
	| 'invoiceFooter'
	| 'numberPadding'
>

export type BillingEvaluation = {
	patch: BillingPatch | null
	errors: Partial<Record<BillingField, string>>
}

const DAYS_HINT = 'Saisis un nombre entier de jours.'
const DIGITS_HINT = 'Saisis un nombre entier de chiffres, par exemple 3.'
const AMOUNT_HINT =
	'Saisis un montant en euros (indemnité forfaitaire), par exemple 40 ou 40,50.'

export function billingFormOf(settings: Settings): BillingForm {
	return {
		paymentTermsDays: String(settings.paymentTermsDays),
		quoteValidityDays: String(settings.quoteValidityDays),
		defaultVatRate: settings.defaultVatRate,
		latePenalty: settings.latePenalty,
		recoveryFee: centsToInput(settings.recoveryFee),
		invoiceFooter: settings.invoiceFooter,
		numberPadding: String(settings.numberPadding),
	}
}

function wholeNumber(input: string): number | null {
	const trimmed = input.trim()
	return /^\d+$/.test(trimmed) ? Number(trimmed) : null
}

function billingFieldOf(message: string): BillingField | null {
	if (message.includes('délai de paiement')) return 'paymentTermsDays'
	if (message.includes('validité')) return 'quoteValidityDays'
	if (message.includes('chiffres du compteur')) return 'numberPadding'
	if (message.includes('recouvrement')) return 'recoveryFee'
	if (message.includes('pénalités')) return 'latePenalty'
	return null
}

export function evaluateBilling(form: BillingForm): BillingEvaluation {
	const errors: Partial<Record<BillingField, string>> = {}
	const paymentTermsDays = wholeNumber(form.paymentTermsDays)
	const quoteValidityDays = wholeNumber(form.quoteValidityDays)
	const numberPadding = wholeNumber(form.numberPadding)
	const recoveryFee = parseAmount(form.recoveryFee)
	if (paymentTermsDays === null) errors.paymentTermsDays = DAYS_HINT
	if (quoteValidityDays === null) errors.quoteValidityDays = DAYS_HINT
	if (numberPadding === null) errors.numberPadding = DIGITS_HINT
	if (recoveryFee === null) errors.recoveryFee = AMOUNT_HINT
	const checked: Parameters<typeof validateSettings>[0] = {
		latePenalty: form.latePenalty.trim(),
		...(paymentTermsDays === null ? {} : { paymentTermsDays }),
		...(quoteValidityDays === null ? {} : { quoteValidityDays }),
		...(numberPadding === null ? {} : { numberPadding }),
		...(recoveryFee === null ? {} : { recoveryFee }),
	}
	for (const message of validateSettings(checked)) {
		const field = billingFieldOf(message)
		if (field !== null && errors[field] === undefined) errors[field] = message
	}
	if (
		Object.keys(errors).length > 0 ||
		paymentTermsDays === null ||
		quoteValidityDays === null ||
		numberPadding === null ||
		recoveryFee === null
	)
		return { patch: null, errors }
	return {
		patch: {
			paymentTermsDays,
			quoteValidityDays,
			defaultVatRate: form.defaultVatRate,
			latePenalty: form.latePenalty.trim(),
			recoveryFee,
			invoiceFooter: form.invoiceFooter.trim(),
			numberPadding,
		},
		errors,
	}
}

export type SequenceRow = {
	kind: SequenceKind
	label: string
	prefix: string
	year: number
	last: number
	lastNumber: string | null
	next: string
}

export type ResumeResult = {
	error: string | null
	nextNumber: number | null
	preview: string | null
}

export function evaluateResume(
	row: SequenceRow,
	input: string,
	padding: number,
): ResumeResult {
	if (input.trim() === '')
		return { error: null, nextNumber: null, preview: null }
	const wanted = wholeNumber(input)
	if (wanted === null || !Number.isSafeInteger(wanted))
		return {
			error: 'Saisis un nombre entier, par exemple 120.',
			nextNumber: null,
			preview: null,
		}
	if (wanted <= row.last + 1)
		return {
			error: `Le prochain numéro est déjà ${row.next} : un compteur ne peut qu’avancer, jamais reculer.`,
			nextNumber: null,
			preview: null,
		}
	return {
		error: null,
		nextNumber: wanted,
		preview: formatNumber(row.prefix, row.year, wanted, padding),
	}
}

export const STRIPE_TYPE_LABELS: Record<StripeEventType, string> = {
	'invoice.paid': 'Facture payée',
	'payment_intent.succeeded': 'Paiement reçu',
	'charge.refunded': 'Remboursement',
}

export const STRIPE_EVENT_HANDLING: readonly {
	type: StripeEventType
	summary: string
}[] = [
	{
		type: 'invoice.paid',
		summary: 'La facture Stripe est importée et rattachée au client.',
	},
	{
		type: 'payment_intent.succeeded',
		summary:
			'Le paiement est affecté à la facture correspondante ou envoyé en rapprochement.',
	},
	{
		type: 'charge.refunded',
		summary: 'Le remboursement est enregistré.',
	},
]

export const SECRET_MASK = 'whsec_••••••••••••'

export function validateSecret(value: string): string | null {
	const secret = value.trim()
	if (secret === '') return 'Saisis le secret de signature.'
	if (!secret.startsWith('whsec_') || secret.length <= 'whsec_'.length)
		return 'Le secret de signature commence par whsec_ et se copie depuis le tableau de bord Stripe.'
	return null
}

export function hasSameValue(a: unknown, b: unknown): boolean {
	return JSON.stringify(a) === JSON.stringify(b)
}
