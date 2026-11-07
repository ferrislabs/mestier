import type { Schemas } from '#/api/api.client'

type Kind = Schemas.CredentialKindResponse
type Field = Schemas.CredentialFieldResponse

export function field(
	name: string,
	label: string,
	overrides: Partial<Field> = {},
): Field {
	return {
		name,
		label,
		kind: 'text',
		secret: false,
		required: false,
		default: null,
		placeholder: null,
		hint: null,
		options: [],
		show_when: null,
		...overrides,
	}
}

export function kind(overrides: Partial<Kind> & Pick<Kind, 'kind'>): Kind {
	return {
		label: overrides.kind,
		category: 'http',
		category_label: 'Requêtes HTTP',
		description: '',
		usages: [{ usage: 'http', label: 'Requête HTTP', has_node: true }],
		origin_choices: [],
		fields: [],
		supported: true,
		executable: true,
		connectable: false,
		...overrides,
	}
}

const HTTP_USAGE = {
	usage: 'http',
	label: 'Requête HTTP et opérations de collection',
	has_node: true,
}
const WEBHOOK_USAGE = { usage: 'webhook', label: 'Webhook', has_node: true }

export const HTTP_BASIC = kind({
	kind: 'http_basic',
	label: 'Basic Auth',
	description:
		'Utilisateur et mot de passe envoyés dans l’en-tête Authorization.',
	usages: [HTTP_USAGE, WEBHOOK_USAGE],
	fields: [
		field('username', 'Utilisateur', { required: true }),
		field('password', 'Mot de passe', {
			kind: 'password',
			secret: true,
			required: true,
		}),
	],
})

export const BEARER_TOKEN = kind({
	kind: 'bearer_token',
	label: 'Bearer',
	description: 'Un jeton envoyé sous la forme « Authorization: Bearer … ».',
	usages: [HTTP_USAGE, WEBHOOK_USAGE],
	origin_choices: ['supplied', 'generated'],
	fields: [
		field('token', 'Jeton', {
			kind: 'password',
			secret: true,
			required: true,
		}),
	],
})

export const HTTP_HEADER = kind({
	kind: 'http_header',
	label: 'Clé d’API dans un en-tête',
	description: 'Une clé transmise dans un en-tête HTTP de votre choix.',
	usages: [HTTP_USAGE, WEBHOOK_USAGE],
	fields: [
		field('header_name', 'Nom de l’en-tête', {
			required: true,
			default: 'X-API-Key',
		}),
		field('header_value', 'Valeur', {
			kind: 'password',
			secret: true,
			required: true,
		}),
	],
})

export const OAUTH2_AUTHORIZATION_CODE = kind({
	kind: 'oauth2_authorization_code',
	label: 'OAuth2',
	description:
		'Connexion déléguée : un utilisateur autorise Mestier à agir sur son compte.',
	usages: [HTTP_USAGE],
	connectable: true,
	fields: [
		field('authorization_url', 'URL d’autorisation', {
			required: true,
			placeholder: 'https://auth.exemple.fr/authorize',
		}),
		field('token_url', 'URL du jeton', {
			required: true,
			placeholder: 'https://auth.exemple.fr/token',
		}),
		field('client_id', 'Client ID', { required: true }),
		field('client_secret', 'Client secret', {
			kind: 'password',
			secret: true,
			required: true,
		}),
		field('scopes', 'Scopes', {
			placeholder: 'openid profile',
			hint: 'Séparés par des espaces.',
		}),
		field('pkce', 'PKCE', {
			kind: 'select',
			default: 'yes',
			options: [
				{ value: 'yes', label: 'Activé (recommandé)' },
				{ value: 'no', label: 'Désactivé' },
			],
		}),
	],
})

export const HMAC = kind({
	kind: 'hmac',
	label: 'Signature HMAC',
	category: 'webhook',
	category_label: 'Webhooks entrants',
	description:
		'Vérifie que les webhooks reçus sont signés avec un secret partagé.',
	usages: [WEBHOOK_USAGE],
	fields: [
		field('header_name', 'En-tête de la signature', {
			required: true,
			default: 'X-Signature',
		}),
		field('algorithm', 'Algorithme', {
			kind: 'select',
			default: 'sha256',
			options: [
				{ value: 'sha256', label: 'SHA-256' },
				{ value: 'sha512', label: 'SHA-512' },
				{ value: 'sha1', label: 'SHA-1' },
			],
		}),
		field('secret', 'Secret partagé', {
			kind: 'password',
			secret: true,
			required: true,
		}),
	],
})

export const SMTP = kind({
	kind: 'smtp',
	label: 'SMTP',
	category: 'email',
	category_label: 'E-mail',
	description: 'Envoi d’e-mails depuis votre serveur de messagerie.',
	usages: [{ usage: 'smtp', label: 'E-mail SMTP', has_node: false }],
	executable: false,
	fields: [
		field('host', 'Hôte', { required: true, placeholder: 'smtp.exemple.fr' }),
		field('port', 'Port', { kind: 'number', required: true, default: '465' }),
		field('security', 'Sécurité', {
			kind: 'select',
			default: 'ssl',
			options: [
				{ value: 'ssl', label: 'SSL/TLS' },
				{ value: 'starttls', label: 'STARTTLS' },
				{ value: 'none', label: 'Aucune' },
			],
		}),
		field('username', 'Utilisateur'),
		field('password', 'Mot de passe', { kind: 'password', secret: true }),
	],
})

export const SSH = kind({
	kind: 'ssh',
	label: 'SSH',
	category: 'servers',
	category_label: 'Serveurs et bases de données',
	description: 'Accès à un serveur par mot de passe ou par clé privée.',
	usages: [{ usage: 'ssh', label: 'Commandes SSH', has_node: false }],
	executable: false,
	fields: [
		field('host', 'Hôte', {
			required: true,
			placeholder: 'serveur.exemple.fr',
		}),
		field('port', 'Port', { kind: 'number', required: true, default: '22' }),
		field('username', 'Utilisateur', { required: true }),
		field('method', 'Méthode', {
			kind: 'select',
			default: 'private-key',
			options: [
				{ value: 'private-key', label: 'Clé privée' },
				{ value: 'password', label: 'Mot de passe' },
			],
		}),
		field('password', 'Mot de passe', {
			kind: 'password',
			secret: true,
			required: true,
			show_when: { field: 'method', one_of: ['password'] },
		}),
		field('private_key', 'Clé privée', {
			kind: 'textarea',
			secret: true,
			required: true,
			placeholder: '-----BEGIN OPENSSH PRIVATE KEY-----',
			show_when: { field: 'method', one_of: ['private-key'] },
		}),
		field('passphrase', 'Phrase de passe', {
			kind: 'password',
			secret: true,
			show_when: { field: 'method', one_of: ['private-key'] },
		}),
	],
})

export const MTLS = kind({
	kind: 'mtls',
	label: 'Certificat client',
	description: 'Authentification mutuelle TLS (mTLS) par certificat.',
	fields: [
		field('certificate', 'Certificat (PEM)', {
			kind: 'textarea',
			required: true,
			placeholder: '-----BEGIN CERTIFICATE-----',
		}),
		field('private_key', 'Clé privée (PEM)', {
			kind: 'textarea',
			secret: true,
			required: true,
		}),
	],
})

export const CUSTOM_AUTH = kind({
	kind: 'custom_auth',
	label: 'Authentification personnalisée',
	description:
		'En-têtes, paramètres d’URL ou champs de corps à ajouter, décrits en JSON.',
	fields: [
		field('json', 'Définition (JSON)', {
			kind: 'json',
			secret: true,
			required: true,
		}),
	],
})

export const SQL = kind({
	kind: 'sql',
	label: 'Base de données SQL',
	category: 'servers',
	category_label: 'Serveurs et bases de données',
	description: 'PostgreSQL, MySQL, MariaDB ou SQL Server.',
	usages: [{ usage: 'database', label: 'Requêtes SQL', has_node: false }],
	executable: false,
	fields: [
		field('engine', 'Moteur', {
			kind: 'select',
			default: 'postgres',
			options: [
				{ value: 'postgres', label: 'PostgreSQL' },
				{ value: 'mysql', label: 'MySQL' },
			],
		}),
		field('host', 'Hôte', { required: true, placeholder: 'db.exemple.fr' }),
		field('port', 'Port', { kind: 'number', required: true, default: '5432' }),
		field('database', 'Base', { required: true }),
	],
})

export const ODOO_API = kind({
	kind: 'odoo_api',
	label: 'Clé d’API Odoo',
	category: 'other',
	category_label: 'Autres',
	description:
		'Accès à une instance Odoo par clé d’API, utilisé par les actions Odoo.',
	usages: [{ usage: 'odoo', label: 'Actions Odoo', has_node: true }],
	fields: [
		field('base_url', 'URL de l’instance', {
			required: true,
			placeholder: 'https://mon-entreprise.odoo.com',
		}),
		field('database', 'Base de données', { required: true }),
		field('username', 'Utilisateur', { required: true }),
		field('api_key', 'Clé d’API', {
			kind: 'password',
			secret: true,
			required: true,
		}),
	],
})

export const KINDS: Kind[] = [
	HTTP_BASIC,
	BEARER_TOKEN,
	HTTP_HEADER,
	OAUTH2_AUTHORIZATION_CODE,
	HMAC,
	SMTP,
	SSH,
	MTLS,
	CUSTOM_AUTH,
	SQL,
	ODOO_API,
]

export function credential(
	overrides: Partial<Schemas.CredentialResponse> = {},
): Schemas.CredentialResponse {
	return {
		id: 'cred-1',
		organization_id: 'org-1',
		kind: 'bearer_token',
		name: 'Signature sortante',
		description: '',
		status: 'valid',
		account: null,
		expires_at: null,
		data: {},
		secrets_set: ['token'],
		used_by_workflows: 0,
		origin: 'supplied',
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}
