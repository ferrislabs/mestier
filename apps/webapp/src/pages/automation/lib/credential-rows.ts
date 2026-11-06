import type {
	Credential,
	CredentialKind,
	CredentialStatus,
} from '#/hooks/use-automation'
import { findKind, kindLabel } from '#/pages/automation/lib/credential-kinds'
import { credentialSummary } from '#/pages/automation/lib/credential-summary'

export interface CredentialRow {
	id: string
	name: string
	description: string
	kind: string
	kindLabel: string
	summary: string
	origin: 'supplied' | 'generated'
	createdAt: string
	updatedAt: string
	usedByWorkflows: number
	status: CredentialStatus
	account: string | null
	data: Readonly<Record<string, unknown>>
	secretsSet: readonly string[]
}

export function credentialRow(
	credential: Credential,
	kinds: readonly CredentialKind[],
): CredentialRow {
	return {
		id: credential.id,
		name: credential.name,
		description: credential.description,
		kind: credential.kind,
		kindLabel: kindLabel(kinds, credential.kind),
		summary: credentialSummary(credential, findKind(kinds, credential.kind)),
		origin: credential.origin === 'generated' ? 'generated' : 'supplied',
		createdAt: credential.created_at,
		updatedAt: credential.updated_at,
		usedByWorkflows: credential.used_by_workflows,
		status: credential.status,
		account: credential.account ?? null,
		data: credential.data,
		secretsSet: credential.secrets_set,
	}
}

export function credentialRows(
	credentials: readonly Credential[],
	kinds: readonly CredentialKind[],
): CredentialRow[] {
	return credentials
		.map((credential) => credentialRow(credential, kinds))
		.sort((a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt))
}
