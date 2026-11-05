export const API_PATH = '/api/v1/support/tickets'

export const EXAMPLE_PAYLOAD = {
	subject: 'Volet roulant bloqué',
	body: 'Le volet du salon ne remonte plus depuis ce matin.',
	requester: {
		name: 'Claire Fontaine',
		email: 'claire.fontaine@fontaine-immo.fr',
	},
	external_id: 'EC-20450',
	priority: 'urgent',
	custom_fields: {
		categorie: 'sav',
		reference_chantier: 'CH-2026-031',
		plateforme: 'web',
	},
}

export function curlExample(origin: string, prefix: string): string {
	return [
		`curl -X POST ${origin}${API_PATH} \\`,
		`  -H "Authorization: Bearer ${prefix}…" \\`,
		'  -H "Content-Type: application/json" \\',
		`  -d '${JSON.stringify(EXAMPLE_PAYLOAD, null, 2).replaceAll("'", "'\\''")}'`,
	].join('\n')
}

export function generateSecret(random: () => number = Math.random): string {
	const alphabet =
		'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
	let secret = 'mst_live_'
	for (let index = 0; index < 32; index += 1)
		secret += alphabet[Math.floor(random() * alphabet.length)]
	return secret
}

export interface ApiPayload {
	subject: string
	body: string
	requesterEmail: string
	requesterName: string | null
	externalId: string | null
}

export type ApiPayloadValidation =
	| { ok: true; payload: ApiPayload }
	| { ok: false; errors: string[] }

function textOf(value: unknown): string | null {
	return typeof value === 'string' && value.trim() !== '' ? value.trim() : null
}

export function validateApiPayload(raw: unknown): ApiPayloadValidation {
	if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) {
		return { ok: false, errors: ['Le corps doit être un objet JSON.'] }
	}

	const record = raw as Record<string, unknown>
	const requester =
		typeof record.requester === 'object' && record.requester !== null
			? (record.requester as Record<string, unknown>)
			: {}
	const errors: string[] = []
	const subject = textOf(record.subject)
	const body = textOf(record.body)
	const email = textOf(requester.email)

	if (!subject) errors.push('subject est obligatoire.')
	if (!body) errors.push('body est obligatoire.')
	if (!email) errors.push('requester.email est obligatoire.')
	else if (!email.includes('@')) errors.push('requester.email est invalide.')

	if (errors.length > 0 || !subject || !body || !email)
		return { ok: false, errors }

	return {
		ok: true,
		payload: {
			subject,
			body,
			requesterEmail: email,
			requesterName: textOf(requester.name),
			externalId: textOf(record.external_id),
		},
	}
}
