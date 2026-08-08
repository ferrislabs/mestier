import type { Schemas } from '#/api/api.client'

export type TaskComment = Schemas.TaskCommentResponse
export type PaginationMetadata = Schemas.PaginationMetadata

/**
 * Whether `comment` was written by the person currently looking at it —
 * drives whether the edit/delete actions render at all (see the planning
 * remodel design doc: the backend answers `403` to anyone else, but this
 * app never lets the user discover that by trying).
 *
 * There is no endpoint that hands the frontend its own internal `UserId`
 * (`TaskCommentAuthorResponse.id` is a Mestier user id, not the OIDC `sub`
 * carried by the auth session — see `find_user_by_sub` in
 * `libs/handlers-planning/src/task_comment/create.rs`), so this compares
 * `display_name` instead of `id`. That is a heuristic, not a proof of
 * identity — two members who happen to share a display name would each see
 * the other's actions, and the backend's own `403` remains the actual
 * guard. Flagged here rather than silently relied upon; see this
 * workstream's report for the missing endpoint this works around.
 */
export function isOwnComment(
	comment: { author: { display_name: string } },
	currentDisplayName: string | null,
): boolean {
	if (currentDisplayName === null) return false
	return comment.author.display_name === currentDisplayName
}

/** Whether a newer page exists — the thread's "load more" control. `null`/`undefined` metadata (not loaded yet) means no. */
export function canLoadMoreComments(
	pagination: PaginationMetadata | null | undefined,
): boolean {
	return pagination?.next_page != null
}

/** Whether an older page exists — proves the thread never fetches its whole history in one request (see this workstream's TDD coverage list). */
export function canLoadOlderComments(
	pagination: PaginationMetadata | null | undefined,
): boolean {
	return pagination?.prev_page != null
}

/**
 * The page a freshly-posted comment lands on, given the thread's pagination
 * *before* the post — oldest-first (see `libs/handlers-planning/src/
 * task_comment/list.rs`) means a new comment is always appended past the
 * last page, not inserted into the first. Without this, posting a comment
 * while looking at an early page would silently succeed with nothing
 * visibly changing, which reads as a bug. `null` metadata (nothing loaded
 * yet, or this is the thread's very first comment) resolves to page 1.
 */
export function nextPageAfterCreate(
	pagination: PaginationMetadata | null | undefined,
	perPage: number,
): number {
	if (!pagination) return 1
	const total = pagination.total ?? 0
	const lastPage = pagination.last_page ?? 1
	return total > 0 && total % perPage === 0 ? lastPage + 1 : lastPage
}
