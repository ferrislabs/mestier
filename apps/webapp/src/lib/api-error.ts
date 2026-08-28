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
