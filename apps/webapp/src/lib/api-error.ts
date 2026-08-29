/**
 * Whether an error thrown by `api.fetch.ts`'s `fetcher` carries a 403.
 *
 * A 403 on a sub-resource (e.g. `employee-profiles` gated on `member.manage`,
 * unlike the plain organization-membership check on `members`) is not a
 * failure — it is a legitimate "you don't have this permission", and a
 * caller that folds every query's `error` into one page-level failure
 * banner ends up reporting an expected access boundary as if the page were
 * broken. See #371.
 */
export function isForbiddenError(error: unknown): boolean {
	return (
		typeof error === 'object' &&
		error !== null &&
		'status' in error &&
		(error as { status?: unknown }).status === 403
	)
}

/**
 * A user-facing message for a mutation error, null when there is none.
 *
 * A 403 is singled out because the API answers it with the raw `"Forbidden"`
 * (`ApiError::Forbidden`'s `#[error(...)]` text, in English, meant for logs)
 * — showing that string as-is under a write control reads as a bug rather
 * than what it is: a permission that was revoked between load and click
 * (#307). Every other failure keeps its own message.
 */
export function mutationErrorMessage(error: unknown): string | null {
	if (error == null) return null
	if (isForbiddenError(error)) {
		return "Vous n'avez plus la permission nécessaire pour cette action."
	}
	return error instanceof Error ? error.message : 'Une erreur est survenue.'
}
