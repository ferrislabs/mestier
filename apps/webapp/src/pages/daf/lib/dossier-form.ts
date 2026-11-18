import type { Schemas } from '#/api/api.client'

export interface DafDossierValues {
	title: string
	issuer: string
	projectOwner: string
	projectManager: string
}

export function dossierToForm(
	dossier: Schemas.DafDossierResponse,
): DafDossierValues {
	return {
		title: dossier.title,
		issuer: dossier.issuer,
		projectOwner: dossier.project_owner ?? '',
		projectManager: dossier.project_manager ?? '',
	}
}

function orNull(value: string): string | null {
	const trimmed = value.trim()
	return trimmed === '' ? null : trimmed
}

export function buildDossierBody(
	values: DafDossierValues,
): Schemas.UpdateDafDossierRequest {
	return {
		title: values.title.trim(),
		issuer: values.issuer.trim(),
		project_owner: orNull(values.projectOwner),
		project_manager: orNull(values.projectManager),
	}
}
