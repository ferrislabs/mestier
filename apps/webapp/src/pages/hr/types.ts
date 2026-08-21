import type { Schemas } from '#/api/api.client'
import type { Rhythm, WorkSlot, WorkTimeRange } from '#/hooks/use-work-time'

export interface MemberFormValues {
	lastName: string
	firstName: string
	hourlyRate: string
	/**
	 * Salaried: costed from {@link monthlyCost} instead of an hourly rate, which
	 * is why the rate field is disabled. It never means the time is free.
	 */
	isSalaried: boolean
	/** Monthly employer cost in euros as typed. Only read when salaried. */
	monthlyCost: string
}

/**
 * The three states the Access column can show for a seat.
 */
export type AccessState = 'none' | 'invited' | 'linkedAccount'

/**
 * `hasPendingInvitation` comes from the organization's pending-invitations
 * list (`usePendingInvitations`), matched by `member_id` — a seat can be
 * targeted by at most one *effective* invitation at a time in the UI (the
 * "Inviter" action is hidden once one is pending), but the backend does not
 * enforce that itself, so this only reflects what the UI offered.
 */
export function accessState(
	member: { account?: unknown },
	hasPendingInvitation: boolean,
): AccessState {
	if (member.account) return 'linkedAccount'
	if (hasPendingInvitation) return 'invited'
	return 'none'
}

// -- Rythme et plages de travail -----------------------------------------

export const WEEKDAYS: { value: number; label: string; short: string }[] = [
	{ value: 1, label: 'Lundi', short: 'Lun' },
	{ value: 2, label: 'Mardi', short: 'Mar' },
	{ value: 3, label: 'Mercredi', short: 'Mer' },
	{ value: 4, label: 'Jeudi', short: 'Jeu' },
	{ value: 5, label: 'Vendredi', short: 'Ven' },
	{ value: 6, label: 'Samedi', short: 'Sam' },
	{ value: 7, label: 'Dimanche', short: 'Dim' },
]

export interface RhythmSlotDraft {
	/** Client-only React key; never sent to the API. */
	key: string
	weekday: number
	startTime: string
	endTime: string
}

export interface RhythmFormValues {
	effectiveFrom: string
	/** Empty string means open-ended (`effective_to: null`). */
	effectiveTo: string
	slots: RhythmSlotDraft[]
}

export interface WorkSlotDraft {
	/** Client-only React key; never sent to the API. */
	key: string
	workDate: string
	startTime: string
	endTime: string
}

export interface WorkSlotsFormValues {
	from: string
	to: string
	slots: WorkSlotDraft[]
}

export interface SlotValidationError {
	key: string
	message: string
}

export interface WeeklyGap {
	plannedMinutes: number
	contractMinutes: number
	deltaMinutes: number
}

function generateDraftKey(): string {
	return typeof crypto !== 'undefined' && 'randomUUID' in crypto
		? crypto.randomUUID()
		: `draft-${Math.random().toString(36).slice(2)}`
}

/**
 * Minutes-from-midnight (`0..1440`, the domain's own range — see
 * `employee_rhythm_slots.ends_minute`) to an `HH:MM` display string. `1440`
 * — end of day — renders as `24:00`, which a native `<input type="time">`
 * cannot hold, hence the hand-rolled text field this feeds.
 */
export function minutesToTime(minutes: number): string {
	const clamped = Math.max(0, Math.min(1440, Math.round(minutes)))
	const hours = Math.floor(clamped / 60)
	const mins = clamped % 60
	return `${String(hours).padStart(2, '0')}:${String(mins).padStart(2, '0')}`
}

/** Inverse of {@link minutesToTime}. Returns `null` on anything unparsable. */
export function timeToMinutes(time: string): number | null {
	const match = /^(\d{1,2}):(\d{2})$/.exec(time.trim())
	if (!match) return null
	const hours = Number(match[1])
	const mins = Number(match[2])
	if (hours < 0 || hours > 24 || mins < 0 || mins > 59) return null
	if (hours === 24 && mins !== 0) return null
	return hours * 60 + mins
}

/**
 * Formats an arbitrary minute duration (not bounded to a single day, unlike
 * {@link minutesToTime}) as `XhYY` — the unit the whole screen uses for
 * durations: weekly contract base, planned totals, the gap between them.
 */
export function formatDurationMinutes(minutes: number): string {
	const sign = minutes < 0 ? '-' : ''
	const abs = Math.round(Math.abs(minutes))
	const hours = Math.floor(abs / 60)
	const mins = abs % 60
	return `${sign}${hours}h${String(mins).padStart(2, '0')}`
}

/** Inverse of {@link formatDurationMinutes} (accepts `XhYY` or bare `Xh`). */
export function parseDurationLabel(value: string): number | null {
	const match = /^(\d{1,4})h(\d{2})?$/i.exec(value.trim())
	if (!match) return null
	const hours = Number(match[1])
	const mins = match[2] ? Number(match[2]) : 0
	if (mins > 59) return null
	return hours * 60 + mins
}

/**
 * The gap between what the rhythm currently plans for a week and the
 * employee's contractual base. Neither is derived from the other — see the
 * "Temps de travail" section of the planning design doc — so this is a
 * comparison, not a correction: a non-zero delta is not an error.
 */
export function computeWeeklyGap(
	slots: { startTime: string; endTime: string }[],
	weeklyContractMinutes: number,
): WeeklyGap {
	const plannedMinutes = slots.reduce((total, slot) => {
		const starts = timeToMinutes(slot.startTime)
		const ends = timeToMinutes(slot.endTime)
		if (starts === null || ends === null || ends <= starts) return total
		return total + (ends - starts)
	}, 0)
	return {
		plannedMinutes,
		contractMinutes: weeklyContractMinutes,
		deltaMinutes: plannedMinutes - weeklyContractMinutes,
	}
}

/** The rhythm version currently in effect, if any (`effective_to == null`). */
export function findOpenRhythm(rhythms: Rhythm[]): Rhythm | null {
	return rhythms.find((rhythm) => rhythm.effective_to == null) ?? null
}

/**
 * `fallbackFrom` seeds the form when the employee has no rhythm yet — the
 * screen defaults to proposing a first version starting today rather than
 * leaving the date blank.
 */
export function rhythmToDraft(
	rhythm: Rhythm | null,
	fallbackFrom: string,
): RhythmFormValues {
	if (!rhythm) {
		return { effectiveFrom: fallbackFrom, effectiveTo: '', slots: [] }
	}
	return {
		effectiveFrom: rhythm.effective_from,
		effectiveTo: rhythm.effective_to ?? '',
		slots: rhythm.slots.map((slot) => ({
			key: generateDraftKey(),
			weekday: slot.weekday,
			startTime: minutesToTime(slot.starts_minute),
			endTime: minutesToTime(slot.ends_minute),
		})),
	}
}

/** Drops any slot whose horaire doesn't parse rather than sending garbage. */
export function draftToRhythmSlots(
	slots: RhythmSlotDraft[],
): Schemas.RhythmSlotRequest[] {
	const result: Schemas.RhythmSlotRequest[] = []
	for (const slot of slots) {
		const starts = timeToMinutes(slot.startTime)
		const ends = timeToMinutes(slot.endTime)
		if (starts === null || ends === null) continue
		result.push({
			weekday: slot.weekday,
			starts_minute: starts,
			ends_minute: ends,
		})
	}
	return result
}

export function emptyRhythmSlotDraft(weekday: number): RhythmSlotDraft {
	return {
		key: generateDraftKey(),
		weekday,
		startTime: '08:00',
		endTime: '12:00',
	}
}

export function workSlotsToDraft(
	workSlots: WorkSlot[],
	range: WorkTimeRange,
): WorkSlotsFormValues {
	const slots = [...workSlots]
		.sort(
			(a, b) =>
				a.work_date.localeCompare(b.work_date) ||
				a.starts_minute - b.starts_minute,
		)
		.map((slot) => ({
			key: slot.id,
			workDate: slot.work_date,
			startTime: minutesToTime(slot.starts_minute),
			endTime: minutesToTime(slot.ends_minute),
		}))
	return { from: range.from, to: range.to, slots }
}

/** Drops any slot whose horaire or date doesn't parse rather than sending garbage. */
export function draftToWorkSlots(
	slots: WorkSlotDraft[],
): Schemas.WorkSlotRequest[] {
	const result: Schemas.WorkSlotRequest[] = []
	for (const slot of slots) {
		const starts = timeToMinutes(slot.startTime)
		const ends = timeToMinutes(slot.endTime)
		if (starts === null || ends === null || !slot.workDate) continue
		result.push({
			work_date: slot.workDate,
			starts_minute: starts,
			ends_minute: ends,
		})
	}
	return result
}

export function emptyWorkSlotDraft(workDate: string): WorkSlotDraft {
	return {
		key: generateDraftKey(),
		workDate,
		startTime: '08:00',
		endTime: '12:00',
	}
}

function validateTimeRange(startTime: string, endTime: string): string | null {
	const starts = timeToMinutes(startTime)
	const ends = timeToMinutes(endTime)
	if (starts === null || ends === null) return 'Horaire invalide'
	if (ends <= starts) return 'La fin doit être après le début'
	return null
}

/**
 * Mirrors the backend's own checks so the form catches them before a
 * round-trip — plus the `effective_from` proactive check the API can't
 * express until it actually rejects the request (see
 * `WorkTimeService::replace_rhythm`'s last branch): starting a version
 * earlier than the one currently open is a 409, so the form warns instead of
 * letting the user hit that wall.
 */
export function validateRhythmDraft(
	values: RhythmFormValues,
	openRhythmEffectiveFrom: string | null,
): SlotValidationError[] {
	const errors: SlotValidationError[] = []
	if (!values.effectiveFrom) {
		errors.push({ key: 'effectiveFrom', message: 'Date de début requise' })
	} else if (
		openRhythmEffectiveFrom &&
		values.effectiveFrom < openRhythmEffectiveFrom
	) {
		errors.push({
			key: 'effectiveFrom',
			message: `Ne peut pas démarrer avant la version en cours (${formatDateFr(openRhythmEffectiveFrom)}). Choisissez cette date ou une date ultérieure.`,
		})
	}
	if (
		values.effectiveTo &&
		values.effectiveFrom &&
		values.effectiveTo <= values.effectiveFrom
	) {
		errors.push({
			key: 'effectiveTo',
			message: 'La date de fin doit être après la date de début',
		})
	}
	for (const slot of values.slots) {
		const message = validateTimeRange(slot.startTime, slot.endTime)
		if (message) errors.push({ key: slot.key, message })
	}
	return errors
}

export function validateWorkSlotsDraft(
	values: WorkSlotsFormValues,
): SlotValidationError[] {
	const errors: SlotValidationError[] = []
	if (!values.from || !values.to) {
		errors.push({ key: 'range', message: 'Période requise' })
	} else if (values.to < values.from) {
		errors.push({
			key: 'range',
			message: 'La date de fin doit être après la date de début',
		})
	}
	for (const slot of values.slots) {
		const message = validateTimeRange(slot.startTime, slot.endTime)
		if (message) {
			errors.push({ key: slot.key, message })
		} else if (
			values.from &&
			values.to &&
			(slot.workDate < values.from || slot.workDate > values.to)
		) {
			errors.push({ key: slot.key, message: 'Hors de la période sélectionnée' })
		}
	}
	return errors
}

export function todayIsoDate(): string {
	const now = new Date()
	const year = now.getFullYear()
	const month = String(now.getMonth() + 1).padStart(2, '0')
	const day = String(now.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
}

export function addDaysIso(date: string, days: number): string {
	const [year, month, day] = date.split('-').map(Number)
	const base = new Date(Date.UTC(year, month - 1, day))
	base.setUTCDate(base.getUTCDate() + days)
	return base.toISOString().slice(0, 10)
}

export function formatDateFr(iso: string): string {
	const date = new Date(`${iso}T00:00:00Z`)
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: '2-digit',
		year: 'numeric',
		timeZone: 'UTC',
	}).format(date)
}

/**
 * No HR screen has an organization timezone to read: `GET
 * /organizations/{id}` doesn't expose `organizations.timezone`, only `GET
 * /planning` does (see the planning design doc), and these screens aren't
 * otherwise coupled to the planning module — fetching a planning window just
 * to read a timezone field would be a strange, wasteful dependency for an
 * HR-only page. Falls back to the browser's own zone; on a device set to a
 * different zone than the organization's configured one, this can disagree
 * slightly with the planning grid (which always resolves through
 * `organizations.timezone`) for non-all-day absences, or shift the
 * displayed day near a midnight boundary — a real, human-reviewable
 * trade-off, not an oversight.
 */
export function browserTimeZone(): string {
	return Intl.DateTimeFormat().resolvedOptions().timeZone
}
