import type {
	Doe,
	DoeForm,
	DoeParty,
	PartyForm,
	UpdateDoeRequest,
} from '#/pages/doe/types'
import { PARTY_ROLES } from '#/pages/doe/types'

function emptyParty(role: PartyForm['role']): PartyForm {
	return { role, name: '', address: '', phone: '', email: '', logoKey: null }
}

function partyToForm(party: DoeParty): PartyForm {
	return {
		role: party.role,
		name: party.name,
		address: party.address ?? '',
		phone: party.phone ?? '',
		email: party.email ?? '',
		logoKey: party.logo_key ?? null,
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

export function buildUpdatePayload(form: DoeForm): UpdateDoeRequest {
	const parties: DoeParty[] = form.parties
		.filter((party) => party.name.trim() !== '')
		.map((party) => ({
			role: party.role,
			name: party.name.trim(),
			address: orNull(party.address),
			phone: orNull(party.phone),
			email: orNull(party.email),
			logo_key: party.logoKey,
		}))

	return {
		title: form.title.trim(),
		place: orNull(form.place),
		lot: orNull(form.lot),
		issued_on: form.issuedOn,
		author: orNull(form.author),
		revision: form.revision.trim(),
		parties,
		sections: form.sections.map((section) => ({
			...section,
			title: section.title.trim(),
		})),
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
