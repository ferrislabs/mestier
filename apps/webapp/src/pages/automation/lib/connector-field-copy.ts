import type { Schemas } from '#/api/api.client'
import { branchLabel } from '#/pages/automation/lib/branch-labels'

export interface FieldCopy {
	label?: string
	defaultValue?: unknown
	placeholder?: string
	hint?: string
	rows?: number
}

export interface DisplayField extends Schemas.FieldResponse {
	defaultValue?: unknown
	placeholder?: string
	hint?: string
	rows?: number
}

const VARIABLE_PLACEHOLDER = 'Glissez une variable depuis la colonne Entrée'
const NAME_PLACEHOLDER = 'Glissez une variable ou saisissez un nom'

const CONTACT_FIELDS: Record<string, FieldCopy> = {
	name: { placeholder: NAME_PLACEHOLDER },
	email: { placeholder: VARIABLE_PLACEHOLDER },
	phone: { placeholder: VARIABLE_PLACEHOLDER },
}

const FIELD_COPY: Record<string, Record<string, FieldCopy>> = {
	'http.request': {
		method: { defaultValue: 'GET' },
		body: { label: 'Corps de la requête' },
	},
	'flow.condition': {
		predicate: {
			placeholder: VARIABLE_PLACEHOLDER,
			hint: `Le flux continue par « ${branchLabel('Then')} » si la condition est remplie, par « ${branchLabel('Else')} » sinon.`,
		},
	},
	'flow.loop': {
		items: { hint: 'La liste à parcourir, ou une expression qui la désigne.' },
	},
	'flow.config': {
		variables: {
			hint: 'Les valeurs que la suite du workflow pourra réutiliser.',
		},
	},
	'mestier.customer.create': CONTACT_FIELDS,
	'mestier.support.assign_ticket': {
		ticket_id: { placeholder: VARIABLE_PLACEHOLDER },
		members: {
			hint: 'Les identifiants des membres à qui assigner le ticket.',
		},
	},
	'mestier.support.add_badge': {
		ticket_id: { placeholder: VARIABLE_PLACEHOLDER },
		label: { placeholder: 'Glissez une variable ou saisissez un libellé' },
		tone: { defaultValue: 'neutral' },
	},
	'odoo.create_partner': CONTACT_FIELDS,
	'odoo.update_partner': {
		partner_id: { placeholder: VARIABLE_PLACEHOLDER },
		...CONTACT_FIELDS,
	},
	'odoo.create_invoice': {
		partner_id: { placeholder: VARIABLE_PLACEHOLDER },
		description: {
			placeholder: 'Glissez une variable ou saisissez un libellé',
		},
		amount: { placeholder: VARIABLE_PLACEHOLDER },
	},
}

const CREDENTIAL_SLOT_AFTER: Record<string, string> = {
	'http.request': 'url',
}

function typographic(label: string): string {
	return label.replaceAll("'", '’')
}

export function displayField(
	kind: string,
	field: Schemas.FieldResponse,
): DisplayField {
	const copy = FIELD_COPY[kind]?.[field.name]
	return {
		...field,
		...copy,
		label: typographic(copy?.label ?? field.label),
	}
}

export function withFieldDefaults(
	kind: string,
	config: Record<string, unknown>,
): Record<string, unknown> | null {
	const missing = Object.entries(FIELD_COPY[kind] ?? {}).filter(
		([name, copy]) =>
			copy.defaultValue !== undefined && config[name] === undefined,
	)
	if (missing.length === 0) return null
	return {
		...config,
		...Object.fromEntries(
			missing.map(([name, copy]) => [name, copy.defaultValue]),
		),
	}
}

export function credentialSlotAfter(kind: string): string | null {
	return CREDENTIAL_SLOT_AFTER[kind] ?? null
}

export function collectionCredentialHint(security: readonly string[]): string {
	const declared =
		security.length > 0
			? `La spec déclare : ${security.join(', ')}.`
			: 'La spec ne déclare pas d’authentification.'
	return `${declared} Aucune identification n’est appliquée automatiquement : choisissez celle à utiliser.`
}
