import type { Schemas } from '#/api/api.client'

export function allowedCredentialKinds(
	auth: Schemas.AuthRequirementResponse,
): string[] {
	if (auth === 'None') return []
	if ('Exactly' in auth) return [auth.Exactly]
	return [...auth.AnyOf]
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
