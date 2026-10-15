import { mutationErrorMessage } from '#/lib/api-error'

function statusOf(error: unknown): number | null {
	if (typeof error !== 'object' || error === null) return null
	const status = (error as { status?: unknown }).status
	return typeof status === 'number' ? status : null
}

/**
 * The backend keeps its messages in English, for logs. What the manager needs
 * to know is what happened to the lines, so each status gets its own sentence.
 */
export function planErrorMessage(error: unknown): string | null {
	if (error == null) return null
	switch (statusOf(error)) {
		case 409:
			return 'Une de ces prestations est déjà planifiée, ou son contrat est archivé ou en brouillon. Rien n’a été créé : le récap est actualisé.'
		case 404:
			return 'Une de ces prestations n’existe plus. Rien n’a été créé : le récap est actualisé.'
		default:
			return mutationErrorMessage(error)
	}
}

export function recapLoadErrorMessage(error: unknown): string | null {
	if (error == null) return null
	if (statusOf(error) === 409) return 'Cette année n’a pas cette semaine.'
	return mutationErrorMessage(error)
}
