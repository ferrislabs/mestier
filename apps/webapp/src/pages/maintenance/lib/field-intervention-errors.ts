import { mutationErrorMessage } from '#/lib/api-error'

/**
 * What a worker on a phone needs to hear when a declared visit is refused.
 * The backend's own messages are English and meant for logs.
 */
export function proposeErrorMessage(error: unknown): string | null {
	if (error == null) return null
	const status =
		typeof error === 'object' ? (error as { status?: unknown }).status : null
	if (status === 409) {
		return 'Cette date est refusée : elle doit tomber dans la saison du contrat et ne pas être après aujourd’hui.'
	}
	if (status === 404) {
		return 'Cette tâche n’est plus une tâche d’entretien qui vous est assignée.'
	}
	return mutationErrorMessage(error)
}
