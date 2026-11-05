import type { AuthScheme } from '#/hooks/use-automation'
import { normalizeText } from '#/pages/automation/lib/text'

export type CredentialStatus = 'valid' | 'expired' | 'not-connected'

export interface CredentialMeta {
	description: string
	status: CredentialStatus
	usedByWorkflows: number
	account?: string
}

export type CredentialUsage = 'http' | 'webhook' | 'smtp'

export type CredentialCategoryId = 'http' | 'webhook'

export const CREDENTIAL_CATEGORIES: readonly {
	id: CredentialCategoryId
	label: string
}[] = [
	{ id: 'http', label: 'Requêtes HTTP' },
	{ id: 'webhook', label: 'Webhooks entrants' },
]

export const USAGE_LABELS: Record<CredentialUsage, string> = {
	http: 'Requête HTTP et opérations de collection',
	webhook: 'Webhook',
	smtp: 'E-mail SMTP',
}

interface KindMeta {
	description: string
	category: CredentialCategoryId
	usages: readonly CredentialUsage[]
}

const KIND_META: Readonly<Record<string, KindMeta>> = {
	http_basic: {
		description:
			'Utilisateur et mot de passe envoyés dans l’en-tête Authorization.',
		category: 'http',
		usages: ['http', 'webhook'],
	},
	bearer_token: {
		description: 'Un jeton envoyé sous la forme « Authorization: Bearer … ».',
		category: 'http',
		usages: ['http'],
	},
	http_header: {
		description: 'Une clé transmise dans un en-tête HTTP de votre choix.',
		category: 'http',
		usages: ['http', 'webhook'],
	},
	odoo_api: {
		description: 'Une clé d’API pour appeler votre instance Odoo.',
		category: 'http',
		usages: ['http'],
	},
}

const FALLBACK_META: Pick<KindMeta, 'category' | 'usages'> = {
	category: 'http',
	usages: ['http'],
}

export function schemeDescription(scheme: AuthScheme): string {
	return (
		KIND_META[scheme.kind]?.description ??
		(scheme.fields.length === 0
			? 'Aucun champ à renseigner.'
			: scheme.fields.map((field) => field.label).join(', '))
	)
}

export function schemeUsageText(kind: string): string {
	const usages = KIND_META[kind]?.usages ?? FALLBACK_META.usages
	return `Utilisable dans : ${usages.map((usage) => USAGE_LABELS[usage]).join(', ')}.`
}

export interface SchemeGroup<T> {
	id: CredentialCategoryId
	label: string
	schemes: readonly T[]
}

export function searchSchemeGroups<
	T extends {
		kind: string
		label: string
		fields: readonly { label: string }[]
	},
>(schemes: readonly T[], query: string): SchemeGroup<T>[] {
	const needle = normalizeText(query)
	return CREDENTIAL_CATEGORIES.map((category) => ({
		...category,
		schemes: schemes.filter((scheme) => {
			const meta = KIND_META[scheme.kind] ?? FALLBACK_META
			if (meta.category !== category.id) return false
			const description =
				KIND_META[scheme.kind]?.description ??
				scheme.fields.map((field) => field.label).join(', ')
			return (
				needle === '' ||
				normalizeText(
					`${scheme.label} ${scheme.kind} ${description} ${category.label}`,
				).includes(needle)
			)
		}),
	})).filter((group) => group.schemes.length > 0)
}

export function isOauthAuthorization(kind: string): boolean {
	return /oauth/.test(kind) && !/client/.test(kind)
}

function hash(value: string): number {
	let result = 0
	for (const char of value) result = (result * 31 + char.charCodeAt(0)) >>> 0
	return result
}

export function seededCredentialMeta(id: string, kind: string): CredentialMeta {
	const seed = hash(id)
	const status: CredentialStatus = isOauthAuthorization(kind)
		? 'not-connected'
		: seed % 5 === 0
			? 'expired'
			: 'valid'
	return { description: '', status, usedByWorkflows: seed % 4 }
}
