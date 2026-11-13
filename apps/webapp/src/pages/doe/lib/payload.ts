import type {
	Doe,
	DoeForm,
	DoeParty,
	PartyForm,
	UpdateDoeRequest,
} from '#/pages/doe/types'
import { PARTY_ROLES } from '#/pages/doe/types'

function emptyParty(role: PartyForm['role']): PartyForm {
	return {
		role,
		name: '',
		address: '',
		phone: '',
		email: '',
		logoKey: null,
		contactName: '',
		contactRole: '',
		contactEmail: '',
		contactPhone: '',
	}
}

function partyToForm(party: DoeParty): PartyForm {
	return {
		role: party.role,
		name: party.name,
		address: party.address ?? '',
		phone: party.phone ?? '',
		email: party.email ?? '',
		logoKey: party.logo_key ?? null,
		contactName: party.contact_name ?? '',
		contactRole: party.contact_role ?? '',
		contactEmail: party.contact_email ?? '',
		contactPhone: party.contact_phone ?? '',
	}
}

export function doeToForm(doe: Doe): DoeForm {
	return {
		title: doe.title,
		place: doe.place ?? '',
		lot: doe.lot ?? '',
		issuedOn: doe.issued_on,
		author: doe.author ?? '',
		revision: doe.revision,
		coverPhotoKey: doe.cover_photo_key ?? null,
		parties: PARTY_ROLES.map((role) => {
			const found = doe.parties.find((party) => party.role === role)
			return found ? partyToForm(found) : emptyParty(role)
		}),
		sections: doe.sections,
	}
}

function orNull(value: string): string | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

/**
 * `expectedUpdatedAt` is the `updated_at` of the DOE the form was loaded
 * from: if someone saved in between (another tab, the AI assistant), the
 * API refuses with 409 instead of the editor overwriting their change.
 */
export function buildUpdatePayload(
	form: DoeForm,
	expectedUpdatedAt: string,
): UpdateDoeRequest {
	const parties: DoeParty[] = form.parties
		.filter((party) => party.name.trim() !== '')
		.map((party) => ({
			role: party.role,
			name: party.name.trim(),
			address: orNull(party.address),
			phone: orNull(party.phone),
			email: orNull(party.email),
			logo_key: party.logoKey,
			contact_name: orNull(party.contactName),
			contact_role: orNull(party.contactRole),
			contact_email: orNull(party.contactEmail),
			contact_phone: orNull(party.contactPhone),
		}))

	return {
		title: form.title.trim(),
		place: orNull(form.place),
		lot: orNull(form.lot),
		issued_on: form.issuedOn,
		author: orNull(form.author),
		revision: form.revision.trim(),
		cover_photo_key: form.coverPhotoKey,
		parties,
		sections: form.sections.map((section) => ({
			...section,
			title: section.title.trim(),
		})),
		expected_updated_at: expectedUpdatedAt,
	}
}

export function isRevisionValid(revision: string): boolean {
	const length = revision.trim().length
	return length >= 1 && length <= 8
}

export function canSave(form: DoeForm): boolean {
	return (
		form.title.trim() !== '' &&
		form.issuedOn !== '' &&
		isRevisionValid(form.revision) &&
		form.sections.every((section) => section.title.trim() !== '')
	)
}

export function formErrors(form: DoeForm): string[] {
	const errors: string[] = []
	if (form.title.trim() === '')
		errors.push('Le titre du chantier est obligatoire.')
	if (form.issuedOn === '') errors.push('La date n’est pas valide.')
	if (!isRevisionValid(form.revision))
		errors.push('L’indice doit faire de 1 à 8 caractères.')
	if (form.sections.some((section) => section.title.trim() === ''))
		errors.push('Chaque thème doit avoir un titre.')
	return errors
}
