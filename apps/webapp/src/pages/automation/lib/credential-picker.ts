import type { Schemas } from '#/api/api.client'

export function allowedCredentialKinds(
	auth: Schemas.AuthRequirementResponse,
): string[] {
	if (auth === 'None') return []
	if ('Exactly' in auth) return [auth.Exactly]
	if ('Optional' in auth) return [...auth.Optional]
	return [...auth.AnyOf]
}

export function isCredentialRequired(
	auth: Schemas.AuthRequirementResponse,
): boolean {
	if (auth === 'None') return false
	return !('Optional' in auth)
}

export function credentialsForAuth(
	credentials: Schemas.CredentialResponse[],
	auth: Schemas.AuthRequirementResponse,
): Schemas.CredentialResponse[] {
	const kinds = allowedCredentialKinds(auth)
	if (kinds.length === 0) return []
	return credentials.filter((credential) => kinds.includes(credential.kind))
}

export function generatedCredentials(
	credentials: Schemas.CredentialResponse[],
): Schemas.CredentialResponse[] {
	return credentials.filter((credential) => credential.origin === 'generated')
}
