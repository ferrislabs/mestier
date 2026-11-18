import type { Schemas } from '#/api/api.client'
import type { DafStatus } from '#/pages/daf/lib/daf-status'

export const MAX_ATTACHMENTS = 20
export const DEFAULT_ATTACHMENT_NAME = 'Pièce jointe'

export interface DafFormValues {
	number: string
	designation: string
	priceRef: string
	supplierName: string
	supplierAddress: string
	supplierPhone: string
	supplierEmail: string
	installer: string
	specs: string
	observations: string
	photoKey: string
	attachments: Schemas.DafAttachment[]
	projectOwner: string
	projectManager: string
	status: DafStatus
	sentOn: string
	issuedOn: string
	returnedOn: string
	reviewNote: string
}

export function dafToForm(daf: Schemas.DafResponse): DafFormValues {
	return {
		number: String(daf.number),
		designation: daf.designation,
		priceRef: daf.price_ref ?? '',
		supplierName: daf.supplier?.name ?? '',
		supplierAddress: daf.supplier?.address ?? '',
		supplierPhone: daf.supplier?.phone ?? '',
		supplierEmail: daf.supplier?.email ?? '',
		installer: daf.installer ?? '',
		specs: daf.specs ?? '',
		observations: daf.observations ?? '',
		photoKey: daf.photo_key ?? '',
		attachments: daf.attachments.map((attachment) => ({ ...attachment })),
		projectOwner: daf.project_owner ?? '',
		projectManager: daf.project_manager ?? '',
		status: daf.status,
		sentOn: daf.sent_on ?? '',
		issuedOn: daf.issued_on,
		returnedOn: daf.returned_on ?? '',
		reviewNote: daf.review_note ?? '',
	}
}

function orNull(value: string): string | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

function supplierOf(form: DafFormValues): Schemas.SupplierSnapshot | null {
	const address = orNull(form.supplierAddress)
	const phone = orNull(form.supplierPhone)
	const email = orNull(form.supplierEmail)
	const name = form.supplierName.trim()
	if (name === '' && !address && !phone && !email) return null
	return { name, address, phone, email }
}

export function buildUpdateBody(
	form: DafFormValues,
	expectedUpdatedAt: string,
): Schemas.UpdateDafRequest {
	return {
		number: Number(form.number),
		designation: form.designation.trim(),
		price_ref: orNull(form.priceRef),
		supplier: supplierOf(form),
		installer: orNull(form.installer),
		specs: orNull(form.specs),
		observations: orNull(form.observations),
		photo_key: orNull(form.photoKey),
		attachments: form.attachments.map((attachment) => ({
			key: attachment.key,
			name: orNull(attachment.name) ?? DEFAULT_ATTACHMENT_NAME,
		})),
		project_owner: orNull(form.projectOwner),
		project_manager: orNull(form.projectManager),
		status: form.status,
		sent_on: orNull(form.sentOn),
		issued_on: form.issuedOn,
		returned_on: orNull(form.returnedOn),
		review_note: orNull(form.reviewNote),
		expected_updated_at: expectedUpdatedAt,
	}
}

export function formErrors(form: DafFormValues): string[] {
	const errors: string[] = []
	const number = Number(form.number)
	if (!Number.isInteger(number) || number < 1 || number > 999) {
		errors.push('Le numéro doit être compris entre 1 et 999.')
	}
	const designation = form.designation.trim()
	if (designation === '' || designation.length > 200) {
		errors.push('La désignation est requise (200 caractères au plus).')
	}
	if (form.issuedOn === '') errors.push('La date d’émission est requise.')
	if (form.attachments.length > MAX_ATTACHMENTS) {
		errors.push(`Au plus ${MAX_ATTACHMENTS} pièces jointes.`)
	}
	const supplier = supplierOf(form)
	if (supplier && supplier.name === '') {
		errors.push('Le nom du fournisseur est requis.')
	}
	return errors
}

export function canSave(form: DafFormValues): boolean {
	return formErrors(form).length === 0
}
