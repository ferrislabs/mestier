import type { Credential, CredentialKind } from '#/hooks/use-automation'

function text(data: Credential['data'], name: string): string {
	const value = data[name]
	return value === undefined || value === null ? '' : String(value)
}

function hostOf(url: string): string {
	if (url === '') return ''
	try {
		return new URL(url).host
	} catch {
		return url
	}
}

function optionLabel(
	kind: CredentialKind | undefined,
	name: string,
	value: string,
): string {
	return (
		kind?.fields
			.find((field) => field.name === name)
			?.options.find((option) => option.value === value)?.label ?? value
	)
}

export function credentialSummary(
	credential: Pick<Credential, 'kind' | 'data' | 'account' | 'origin'>,
	kind?: CredentialKind,
): string {
	const { data } = credential
	const address = `${text(data, 'host')}${
		text(data, 'port') ? `:${text(data, 'port')}` : ''
	}`
	if (credential.origin === 'generated') return 'Générée par Mestier'
	switch (credential.kind) {
		case 'http_basic':
		case 'digest':
			return `Utilisateur ${text(data, 'username')}`
		case 'bearer_token':
			return 'En-tête Authorization: Bearer'
		case 'http_header':
			return `En-tête ${text(data, 'header_name')}`
		case 'api_key_query':
			return `Paramètre ${text(data, 'param_name')}`
		case 'oauth2_authorization_code':
		case 'oauth1':
			return credential.account
				? `Connecté : ${credential.account}`
				: 'Compte non connecté'
		case 'oauth2_client_credentials':
			return `Jeton via ${hostOf(text(data, 'token_url'))}`
		case 'jwt':
			return `Signature ${text(data, 'algorithm')}`
		case 'mtls':
			return 'Certificat et clé privée'
		case 'custom_auth':
			return 'Définition JSON'
		case 'hmac':
			return `En-tête ${text(data, 'header_name')} · ${optionLabel(kind, 'algorithm', text(data, 'algorithm'))}`
		case 'ssh':
			return `${text(data, 'username')}@${address}`
		case 'sql':
			return `${optionLabel(kind, 'engine', text(data, 'engine'))} · ${address}/${text(data, 'database')}`
		case 'ftp':
			return `${optionLabel(kind, 'protocol', text(data, 'protocol'))} · ${address}`
		case 'odoo_api':
			return `${hostOf(text(data, 'base_url'))} · ${text(data, 'database')}`
		default:
			return address || 'Fournie'
	}
}
