import { TZDate } from '@date-fns/tz'
import { addDays, differenceInCalendarDays, parseISO } from 'date-fns'
import type { Schemas } from '#/api/api.client'

export type AssigneeRef = Schemas.AssigneeRefRequest
export type UpdateWorkOrderRequest = Schemas.UpdateWorkOrderRequest

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

export interface WorkOrderDropInput {
	source: DragSource
	target: DropTarget
	entry: { startsAt: string; endsAt: string; employeeIds: string[] }
	timeZone: string
}

export interface WorkOrderPatchResult {
	changed: boolean
	body: UpdateWorkOrderRequest
}

/**
 * `employee:<uuid>` / `member:<uuid>` → the assignee ref the `PATCH` payload
 * expects. `GridResourceRowVM.resourceId` is verbatim the API's
 * `resource_id` (see the planning design doc), so any other shape is a bug
 * upstream, not data to tolerate — this throws rather than guessing.
 */
export function assigneeRefFromResourceId(resourceId: string): AssigneeRef {
	const separatorIndex = resourceId.indexOf(':')
	if (separatorIndex === -1) {
		throw new Error(`resource_id inattendu : ${resourceId}`)
	}
	const kind = resourceId.slice(0, separatorIndex)
	const id = resourceId.slice(separatorIndex + 1)
	if (kind === 'employee') return { kind: 'employee', employee_id: id }
	if (kind === 'member') return { kind: 'member', user_id: id }
	throw new Error(`resource_id inattendu : ${resourceId}`)
}

function assigneeKey(ref: AssigneeRef): string {
	return ref.kind === 'employee'
		? `employee:${ref.employee_id}`
		: `member:${ref.user_id}`
}

/**
 * Replaces `sourceResourceId`'s assignment with `targetResourceId`'s inside
 * `employeeIds`, preserving every other assignee and their order — the
 * complete list a drop's `PATCH` sends, never a delta (see the planning
 * design doc's "Drag & drop" decision). Dropping onto a resource already on
 * the work order collapses to one assignment instead of duplicating it.
 */
export function buildAssigneesForMove(
	employeeIds: string[],
	sourceResourceId: string,
	targetResourceId: string,
): AssigneeRef[] {
	const targetRef = assigneeRefFromResourceId(targetResourceId)
	const sourceKey = assigneeRefFromResourceId(sourceResourceId)
	const sourceKeyString = assigneeKey(sourceKey)
	const targetKeyString = assigneeKey(targetRef)

	const seen = new Set<string>()
	const result: AssigneeRef[] = []

	for (const employeeId of employeeIds) {
		const currentRef: AssigneeRef = {
			kind: 'employee',
			employee_id: employeeId,
		}
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
	employeeIds: string[],
	resourceId: string,
): AssigneeRef[] {
	const targetKeyString = assigneeKey(assigneeRefFromResourceId(resourceId))
	return employeeIds
		.filter(
			(employeeId) =>
				assigneeKey({ kind: 'employee', employee_id: employeeId }) !==
				targetKeyString,
		)
		.map(
			(employeeId) =>
				({ kind: 'employee', employee_id: employeeId }) as AssigneeRef,
		)
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
export function computeWorkOrderDropPatch(
	input: WorkOrderDropInput,
): WorkOrderPatchResult {
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
		entry.employeeIds,
		source.resourceId,
		target.resourceId,
	)
	const body: UpdateWorkOrderRequest = { assignees }
	if (dayDelta !== 0) {
		body.starts_at = shiftInstant(entry.startsAt, dayDelta, timeZone)
		body.ends_at = shiftInstant(entry.endsAt, dayDelta, timeZone)
	}

	return { changed: true, body }
}

/**
 * The `PATCH` body that unassigns one resource from a work order — reuses
 * {@link buildAssigneesForRemoval} so a removal travels the same
 * complete-list path as a move.
 */
export function computeRemoveAssigneePatch(input: {
	employeeIds: string[]
	resourceId: string
}): WorkOrderPatchResult {
	const assignees = buildAssigneesForRemoval(
		input.employeeIds,
		input.resourceId,
	)
	if (assignees.length === input.employeeIds.length) {
		return { changed: false, body: {} }
	}
	return { changed: true, body: { assignees } }
}
