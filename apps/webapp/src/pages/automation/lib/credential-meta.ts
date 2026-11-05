import type { AuthField, AuthScheme } from '#/hooks/use-automation'
import { normalizeText } from '#/pages/automation/lib/text'

export type CredentialStatus = 'valid' | 'expired' | 'not-connected'

export interface CredentialMeta {
	description: string
	status: CredentialStatus
	usedByWorkflows: number
	account?: string
}

export type CredentialUsage = 'http' | 'webhook' | 'smtp' | 'odoo'

export type CredentialCategoryId = 'http' | 'webhook' | 'email' | 'servers'

export const CREDENTIAL_CATEGORIES: readonly {
	id: CredentialCategoryId
	label: string
}[] = [
	{ id: 'http', label: 'Requêtes HTTP' },
	{ id: 'webhook', label: 'Webhooks entrants' },
	{ id: 'email', label: 'E-mail' },
	{ id: 'servers', label: 'Serveurs et bases de données' },
]

export const USAGE_LABELS: Record<CredentialUsage, string> = {
	http: 'Requête HTTP et opérations de collection',
	webhook: 'Webhook',
	smtp: 'E-mail SMTP',
	odoo: 'Actions Odoo (partenaires, factures)',
}

export const USAGES_WITH_NODES: ReadonlySet<CredentialUsage> = new Set([
	'http',
	'odoo',
])

interface FieldMeta {
	label: string
	defaultValue?: string
	placeholder?: string
	hint?: string
}

interface KindMeta {
	label: string
	description: string
	category: CredentialCategoryId
	usages: readonly CredentialUsage[]
	fields: Readonly<Record<string, FieldMeta>>
}

const KIND_META: Readonly<Record<string, KindMeta>> = {
	http_basic: {
		label: 'Basic Auth',
		description:
			'Utilisateur et mot de passe envoyés dans l’en-tête Authorization.',
		category: 'http',
		usages: ['http', 'webhook'],
		fields: {
			username: { label: 'Utilisateur' },
			password: { label: 'Mot de passe' },
		},
	},
	bearer_token: {
		label: 'Bearer',
		description: 'Un jeton envoyé sous la forme « Authorization: Bearer … ».',
		category: 'http',
		usages: ['http'],
		fields: {
			token: { label: 'Jeton' },
		},
	},
	http_header: {
		label: 'Clé d’API dans un en-tête',
		description: 'Une clé transmise dans un en-tête HTTP de votre choix.',
		category: 'http',
		usages: ['http', 'webhook'],
		fields: {
			header_name: { label: 'Nom de l’en-tête', defaultValue: 'X-API-Key' },
			header_value: { label: 'Valeur' },
		},
	},
	odoo_api: {
		label: 'Clé d’API Odoo',
		description: 'Une clé d’API pour appeler votre instance Odoo.',
		category: 'http',
		usages: ['odoo'],
		fields: {
			base_url: {
				label: 'URL de l’instance',
				placeholder: 'https://exemple.odoo.com',
			},
			database: { label: 'Base de données', placeholder: 'exemple' },
			username: { label: 'Utilisateur', placeholder: 'contact@exemple.fr' },
			api_key: { label: 'Clé d’API' },
		},
	},
}

const FALLBACK_META: Pick<KindMeta, 'category' | 'usages'> = {
	category: 'http',
	usages: ['http'],
}

export interface LocalizedField extends AuthField {
	defaultValue?: string
	placeholder?: string
	hint?: string
}

export interface LocalizedScheme extends AuthScheme {
	fields: LocalizedField[]
}

export function localizeScheme(scheme: AuthScheme): LocalizedScheme {
	const meta = KIND_META[scheme.kind]
	return {
		...scheme,
		label: meta?.label ?? scheme.label,
		fields: scheme.fields.map((field) => ({
			...field,
			...meta?.fields[field.name],
		})),
	}
}

export function kindLabel(kind: string, serverLabel?: string): string {
	return KIND_META[kind]?.label ?? serverLabel ?? kind
}

interface SchemeLike {
	kind: string
	label: string
	fields: readonly { label: string }[]
}

export function schemeDescription(scheme: SchemeLike): string {
	return (
		KIND_META[scheme.kind]?.description ??
		(scheme.fields.length === 0
			? 'Aucun champ à renseigner.'
			: scheme.fields.map((field) => field.label).join(', '))
	)
}

export function usageSentence(usages: readonly CredentialUsage[]): string {
	const names = (list: readonly CredentialUsage[]) =>
		list.map((usage) => USAGE_LABELS[usage]).join(', ')
	const usable = usages.filter((usage) => USAGES_WITH_NODES.has(usage))
	return usable.length > 0
		? `Utilisable dans : ${names(usable)}.`
		: `Prévue pour : ${names(usages)}. Aucun nœud ne l’utilise encore.`
}

export function schemeUsageText(kind: string): string {
	return usageSentence(KIND_META[kind]?.usages ?? FALLBACK_META.usages)
}

export interface SchemeGroup<T> {
	id: CredentialCategoryId
	label: string
	schemes: readonly T[]
}

export function searchSchemeGroups<T extends SchemeLike>(
	schemes: readonly T[],
	query: string,
): SchemeGroup<T>[] {
	const needle = normalizeText(query)
	return CREDENTIAL_CATEGORIES.map((category) => ({
		...category,
		schemes: schemes.filter((scheme) => {
			const meta = KIND_META[scheme.kind] ?? FALLBACK_META
			if (meta.category !== category.id) return false
			return (
				needle === '' ||
				normalizeText(
					`${kindLabel(scheme.kind, scheme.label)} ${schemeDescription(scheme)} ${category.label}`,
				).includes(needle)
			)
		}),
	})).filter((group) => group.schemes.length > 0)
}

export function newestFirst<T extends { created_at: string }>(
	credentials: readonly T[],
): T[] {
	return [...credentials].sort(
		(a, b) => Date.parse(b.created_at) - Date.parse(a.created_at),
	)
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
