import { TZDate } from '@date-fns/tz'
import { addDays, differenceInCalendarDays, parseISO } from 'date-fns'
import type { Schemas } from '#/api/api.client'

export type AssigneeRef = Schemas.AssigneeRefRequest
export type UpdateTaskRequest = Schemas.UpdateTaskRequest

/** Where a drag started: the entry being moved, the row it was rendered on, and the day column it sat in. */
export interface DragSource {
	entryId: string
	resourceId: string
	date: string
}

/** Where it landed: the row and day column of the cell the drop happened on. */
export interface DropTarget {
	resourceId: string
	date: string
}

export interface TaskDropInput {
	source: DragSource
	target: DropTarget
	entry: { startsAt: string; endsAt: string; memberIds: string[] }
	timeZone: string
}

export interface TaskPatchResult {
	changed: boolean
	body: UpdateTaskRequest
}

/**
 * `member:<uuid>` → the assignee ref the `PATCH` payload expects.
 * `GridResourceRowVM.resourceId` is verbatim the API's `resource_id` (see
 * the planning design doc — every plannable resource is a member, so
 * `resource_id()` is unconditionally `"member:{uuid}"`), so any other shape
 * is a bug upstream, not data to tolerate — this throws rather than
 * guessing.
 */
export function assigneeRefFromResourceId(resourceId: string): AssigneeRef {
	const separatorIndex = resourceId.indexOf(':')
	if (
		separatorIndex === -1 ||
		resourceId.slice(0, separatorIndex) !== 'member'
	) {
		throw new Error(`resource_id inattendu : ${resourceId}`)
	}
	return { member_id: resourceId.slice(separatorIndex + 1) }
}

/**
 * The inverse of {@link assigneeRefFromResourceId} — an `AssigneeRef` back
 * to the `member:<uuid>` resource id form the grid and the assignee picker
 * both key on. Also the identity every dedup in this file keys on
 * (`buildAssigneesForMove`/`buildAssigneesForRemoval`), so it is exported
 * under one name instead of living twice.
 */
export function resourceIdFromAssigneeRef(ref: AssigneeRef): string {
	return `member:${ref.member_id}`
}

function assigneeKey(ref: AssigneeRef): string {
	return resourceIdFromAssigneeRef(ref)
}

/**
 * Adds `resourceId` to `assignees` if absent, removes it if present —
 * never mutates `assignees`. The task form's assignee picker toggle: unlike
 * a grid drop (`buildAssigneesForMove`), there is no "source" resource to
 * replace, only a set to grow or shrink.
 */
export function toggleAssignee(
	assignees: AssigneeRef[],
	resourceId: string,
): AssigneeRef[] {
	const exists = assignees.some(
		(ref) => resourceIdFromAssigneeRef(ref) === resourceId,
	)
	if (exists) {
		return assignees.filter(
			(ref) => resourceIdFromAssigneeRef(ref) !== resourceId,
		)
	}
	return [...assignees, assigneeRefFromResourceId(resourceId)]
}

/**
 * Replaces `sourceResourceId`'s assignment with `targetResourceId`'s inside
 * `memberIds`, preserving every other assignee and their order — the
 * complete list a drop's `PATCH` sends, never a delta (see the planning
 * design doc's "Drag & drop" decision). Dropping onto a resource already on
 * the task collapses to one assignment instead of duplicating it.
 */
export function buildAssigneesForMove(
	memberIds: string[],
	sourceResourceId: string,
	targetResourceId: string,
): AssigneeRef[] {
	const targetRef = assigneeRefFromResourceId(targetResourceId)
	const sourceKey = assigneeRefFromResourceId(sourceResourceId)
	const sourceKeyString = assigneeKey(sourceKey)
	const targetKeyString = assigneeKey(targetRef)

	const seen = new Set<string>()
	const result: AssigneeRef[] = []

	for (const memberId of memberIds) {
		const currentRef: AssigneeRef = { member_id: memberId }
		const currentKeyString = assigneeKey(currentRef)
		const isSource = currentKeyString === sourceKeyString
		const outputRef = isSource ? targetRef : currentRef
		const outputKeyString = isSource ? targetKeyString : currentKeyString
		if (seen.has(outputKeyString)) continue
		seen.add(outputKeyString)
		result.push(outputRef)
	}

	if (!seen.has(targetKeyString)) {
		result.push(targetRef)
	}

	return result
}

/**
 * Drops `resourceId`'s assignment, keeping every other assignee untouched —
 * the same complete-list path a move uses (see the planning design doc:
 * "l'ajout comme le retrait passent par le même chemin").
 */
export function buildAssigneesForRemoval(
	memberIds: string[],
	resourceId: string,
): AssigneeRef[] {
	const targetKeyString = assigneeKey(assigneeRefFromResourceId(resourceId))
	return memberIds
		.filter(
			(memberId) => assigneeKey({ member_id: memberId }) !== targetKeyString,
		)
		.map((memberId) => ({ member_id: memberId }) as AssigneeRef)
}

/**
 * Shifts an instant by whole calendar days, computed in `timeZone` so the
 * shift preserves wall-clock time across a DST change instead of a fixed
 * `days * 24h` offset. `TZDate#toISOString` renders with its own zone's
 * offset (correct, but not the `Z`-suffixed form the rest of the API traffic
 * uses) — wrapping the result in a plain `Date` before formatting normalizes
 * it, without touching the underlying instant.
 */
export function shiftInstant(
	iso: string,
	days: number,
	timeZone: string,
): string {
	if (days === 0) return iso
	const zoned = new TZDate(iso, timeZone)
	const shifted = addDays(zoned, days)
	return new Date(shifted.getTime()).toISOString()
}

/**
 * The single `PATCH` body a drag & drop gesture produces — see the planning
 * design doc's "Drag & drop" decision: one call whether the gesture changes
 * the day, the row, or both. `assignees` is always the complete list
 * whenever anything changes, even when the row is unchanged, so the request
 * shape never depends on which axis moved. `changed: false` for a drop that
 * lands back where it started — the caller skips both the call and the
 * confirmation dialog.
 */
export function computeTaskDropPatch(input: TaskDropInput): TaskPatchResult {
	const { source, target, entry, timeZone } = input
	const dayDelta = differenceInCalendarDays(
		parseISO(target.date),
		parseISO(source.date),
	)
	const resourceChanged = target.resourceId !== source.resourceId

	if (dayDelta === 0 && !resourceChanged) {
		return { changed: false, body: {} }
	}

	const assignees = buildAssigneesForMove(
		entry.memberIds,
		source.resourceId,
		target.resourceId,
	)
	const body: UpdateTaskRequest = { assignees }
	if (dayDelta !== 0) {
		body.starts_at = shiftInstant(entry.startsAt, dayDelta, timeZone)
		body.ends_at = shiftInstant(entry.endsAt, dayDelta, timeZone)
	}

	return { changed: true, body }
}

/**
 * The `PATCH` body that unassigns one resource from a task — reuses
 * {@link buildAssigneesForRemoval} so a removal travels the same
 * complete-list path as a move.
 */
export function computeRemoveAssigneePatch(input: {
	memberIds: string[]
	resourceId: string
}): TaskPatchResult {
	const assignees = buildAssigneesForRemoval(input.memberIds, input.resourceId)
	if (assignees.length === input.memberIds.length) {
		return { changed: false, body: {} }
	}
	return { changed: true, body: { assignees } }
}
