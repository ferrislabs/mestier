import { AlertCircle, UserX } from 'lucide-react'
import { useState } from 'react'
import {
	useAbsences,
	useCreateAbsence,
	useDeleteAbsence,
	useUpdateAbsence,
} from '#/hooks/use-absences'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useReferenceCatalog,
	useUpdateEmployee,
} from '#/hooks/use-reference-catalog'
import type { Rhythm } from '#/hooks/use-work-time'
import {
	useReplaceRhythm,
	useReplaceWorkSlots,
	useWorkTime,
} from '#/hooks/use-work-time'
import {
	type AbsenceFormValues,
	type AbsenceListItem,
	absenceToDraft,
	draftToCreateAbsenceRequest,
	draftToUpdateAbsenceRequest,
	emptyAbsenceDraft,
	validateAbsenceDraft,
} from '#/pages/hr/lib/absences'
import {
	addDaysIso,
	computeWeeklyGap,
	draftToRhythmSlots,
	draftToWorkSlots,
	employeeDisplayName,
	emptyRhythmSlotDraft,
	emptyWorkSlotDraft,
	findOpenRhythm,
	formatDateFr,
	formatDurationMinutes,
	parseDurationLabel,
	type RhythmFormValues,
	type RhythmSlotDraft,
	rhythmToDraft,
	todayIsoDate,
	validateRhythmDraft,
	validateWorkSlotsDraft,
	type WorkSlotDraft,
	type WorkSlotsFormValues,
	workSlotsToDraft,
} from '#/pages/hr/types'
import type { AbsenceEmployeeOption } from '#/pages/hr/ui/absence-form-sheet'
import { EmployeeWorkTimeUI } from '#/pages/hr/ui/employee-work-time-ui'

const WORK_SLOTS_WINDOW_DAYS = 13

interface AbsenceSheetState {
	mode: 'create' | 'edit'
	absenceId: string | null
	draft: AbsenceFormValues
}

/**
 * This screen has no organization timezone to read: `GET
 * /organizations/{id}` doesn't expose `organizations.timezone`, only `GET
 * /planning` does (see the planning design doc), and this screen isn't
 * otherwise coupled to the planning module — fetching a planning window just
 * to read a timezone field would be a strange, wasteful dependency for an
 * HR-only page. Falls back to the browser's own zone; on a device set to a
 * different zone than the organization's configured one, this can disagree
 * slightly with the planning grid (which always resolves through
 * `organizations.timezone`) for non-all-day absences, or shift the
 * displayed day near a midnight boundary — a real, human-reviewable
 * trade-off, not an oversight.
 */
function browserTimeZone(): string {
	return Intl.DateTimeFormat().resolvedOptions().timeZone
}

interface EmployeeWorkTimeFeatureProps {
	employeeId: string
}

export function EmployeeWorkTimeFeature({
	employeeId,
}: EmployeeWorkTimeFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le temps de travail nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<EmployeeWorkTimeScreen
			key={`${activeOrganization.id}:${employeeId}`}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			employeeId={employeeId}
		/>
	)
}

interface EmployeeWorkTimeScreenProps {
	organizationId: string
	organizationName: string
	employeeId: string
}

function EmployeeWorkTimeScreen({
	organizationId,
	organizationName,
	employeeId,
}: EmployeeWorkTimeScreenProps) {
	const catalog = useReferenceCatalog(organizationId, {
		equipment: false,
		serviceRates: false,
		products: false,
	})
	const employee =
		catalog.employees.data?.data.find((item) => item.id === employeeId) ?? null

	const [today] = useState(() => todayIsoDate())
	const todayRange = { from: today, to: today }
	const [workSlotsRange, setWorkSlotsRange] = useState(() => ({
		from: today,
		to: addDaysIso(today, WORK_SLOTS_WINDOW_DAYS),
	}))

	// The rhythm in effect "today" must not depend on whichever window the
	// work-slots section happens to be browsing — a separate, narrow read
	// keeps the two independent (see the planning design doc's "Rythme vs
	// plages de travail" section).
	const currentRhythmQuery = useWorkTime(organizationId, employeeId, todayRange)
	const workTimeQuery = useWorkTime(organizationId, employeeId, workSlotsRange)

	const updateEmployee = useUpdateEmployee()
	const replaceRhythm = useReplaceRhythm(employeeId)
	const replaceWorkSlots = useReplaceWorkSlots(employeeId)

	const openRhythm = findOpenRhythm(currentRhythmQuery.data?.data.rhythms ?? [])
	const otherRhythms = dedupeRhythms(
		[
			...(currentRhythmQuery.data?.data.rhythms ?? []),
			...(workTimeQuery.data?.data.rhythms ?? []),
		],
		openRhythm,
	)

	const [rhythmDraft, setRhythmDraft] = useState<RhythmFormValues | null>(null)
	const rhythmValues = rhythmDraft ?? rhythmToDraft(openRhythm, today)

	const [workSlotsDraft, setWorkSlotsDraft] =
		useState<WorkSlotsFormValues | null>(null)
	const workSlotsValues =
		workSlotsDraft ??
		workSlotsToDraft(workTimeQuery.data?.data.work_slots ?? [], workSlotsRange)

	const [contractDraft, setContractDraft] = useState<string | null>(null)

	const absencesQuery = useAbsences(organizationId)
	const createAbsence = useCreateAbsence()
	const updateAbsence = useUpdateAbsence()
	const deleteAbsence = useDeleteAbsence()
	const [absenceSheet, setAbsenceSheet] = useState<AbsenceSheetState | null>(
		null,
	)

	if (catalog.employees.isLoading) {
		return <EmployeeWorkTimeUI.Loading />
	}

	if (!employee) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<UserX className="size-6 text-muted-foreground" />
				</div>
				<div>
					<p className="font-semibold">Employé introuvable</p>
					<p className="text-sm text-muted-foreground">
						Aucun employé ne correspond à cet identifiant dans cette
						organisation.
					</p>
				</div>
			</div>
		)
	}

	const contractValue =
		contractDraft ?? formatDurationMinutes(employee.weekly_contract_minutes)
	const contractMinutesForGap =
		parseDurationLabel(contractValue) ?? employee.weekly_contract_minutes
	const weeklyGap = computeWeeklyGap(rhythmValues.slots, contractMinutesForGap)
	const contractParseError =
		contractDraft !== null && parseDurationLabel(contractDraft) === null
			? 'Format attendu : « 35h00 » ou « 35h ».'
			: null

	const patchRhythmDraft = (patch: Partial<RhythmFormValues>) => {
		const base = rhythmDraft ?? rhythmToDraft(openRhythm, today)
		setRhythmDraft({ ...base, ...patch })
	}

	const patchWorkSlotsDraft = (patch: Partial<WorkSlotsFormValues>) => {
		const base =
			workSlotsDraft ??
			workSlotsToDraft(
				workTimeQuery.data?.data.work_slots ?? [],
				workSlotsRange,
			)
		setWorkSlotsDraft({ ...base, ...patch })
	}

	const rhythmErrors = validateRhythmDraft(
		rhythmValues,
		openRhythm?.effective_from ?? null,
	)
	const workSlotsErrors = validateWorkSlotsDraft(workSlotsValues)

	const rhythmSaveError = replaceRhythm.error
		? errorStatus(replaceRhythm.error) === 409
			? rhythmConflictMessage(openRhythm?.effective_from ?? null)
			: replaceRhythm.error.message
		: null

	const handleUpdateContract = async () => {
		const minutes = parseDurationLabel(contractValue)
		if (minutes === null) return
		try {
			await updateEmployee.mutateAsync({
				path: { employee_id: employee.id },
				body: {
					last_name: employee.last_name,
					first_name: employee.first_name,
					hourly_rate_cents: employee.hourly_rate_cents ?? null,
					user_id: employee.user_id ?? null,
					weekly_contract_minutes: minutes,
				},
			})
			setContractDraft(null)
		} catch {
			// Surfaced reactively via `updateEmployee.error` — the draft is kept
			// so the user doesn't lose what they typed.
		}
	}

	const handleSubmitRhythm = async () => {
		if (rhythmErrors.length > 0) return
		try {
			await replaceRhythm.mutateAsync({
				path: { organization_id: organizationId, employee_id: employeeId },
				body: {
					effective_from: rhythmValues.effectiveFrom,
					effective_to: rhythmValues.effectiveTo || null,
					slots: draftToRhythmSlots(rhythmValues.slots),
				},
			})
			setRhythmDraft(null)
		} catch {
			// Surfaced reactively via `rhythmSaveError` — the draft is kept so a
			// 409 (see `rhythmConflictMessage`) doesn't discard the user's edits.
		}
	}

	const handleSubmitWorkSlots = async () => {
		if (workSlotsErrors.length > 0) return
		try {
			await replaceWorkSlots.mutateAsync({
				path: { organization_id: organizationId, employee_id: employeeId },
				query: { from: workSlotsValues.from, to: workSlotsValues.to },
				body: { slots: draftToWorkSlots(workSlotsValues.slots) },
			})
			setWorkSlotsRange({ from: workSlotsValues.from, to: workSlotsValues.to })
			setWorkSlotsDraft(null)
		} catch {
			// Surfaced reactively via `replaceWorkSlots.error` — the draft is
			// kept so the user doesn't lose what they typed.
		}
	}

	// See `browserTimeZone`'s doc for why this isn't `organizations.timezone`.
	const timeZone = browserTimeZone()

	const employeeOptions: AbsenceEmployeeOption[] = (
		catalog.employees.data?.data ?? []
	).map((candidate) => ({
		employeeId: candidate.id,
		displayName: employeeDisplayName(candidate),
	}))

	const employeeAbsences: AbsenceListItem[] = (absencesQuery.data?.data ?? [])
		.filter((absence) => absence.employee_id === employeeId)
		.map((absence) => ({
			id: absence.id,
			...absenceToDraft({ ...absence, absence_kind: absence.kind }, timeZone),
		}))
		.sort((a, b) => a.range.from.localeCompare(b.range.from))

	function handleCreateAbsence() {
		setAbsenceSheet({
			mode: 'create',
			absenceId: null,
			draft: emptyAbsenceDraft(employeeId, today),
		})
	}

	function handleSelectAbsence(absenceId: string) {
		const absence = (absencesQuery.data?.data ?? []).find(
			(candidate) => candidate.id === absenceId,
		)
		if (!absence) return
		setAbsenceSheet({
			mode: 'edit',
			absenceId,
			draft: absenceToDraft(
				{ ...absence, absence_kind: absence.kind },
				timeZone,
			),
		})
	}

	function handleAbsenceSheetOpenChange(open: boolean) {
		if (!open) setAbsenceSheet(null)
	}

	function handleAbsenceDraftChange(patch: Partial<AbsenceFormValues>) {
		setAbsenceSheet((current) =>
			current ? { ...current, draft: { ...current.draft, ...patch } } : current,
		)
	}

	async function handleSubmitAbsence() {
		if (!absenceSheet) return

		if (absenceSheet.mode === 'create') {
			const request = draftToCreateAbsenceRequest(absenceSheet.draft, timeZone)
			if (!request) return
			try {
				await createAbsence.mutateAsync({
					path: { organization_id: organizationId },
					body: request,
				})
				setAbsenceSheet(null)
			} catch {
				// Surfaced reactively via `createAbsence.error` — the draft is kept.
			}
			return
		}

		if (!absenceSheet.absenceId) return
		const request = draftToUpdateAbsenceRequest(absenceSheet.draft, timeZone)
		if (!request) return
		try {
			await updateAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: absenceSheet.absenceId,
				},
				body: request,
			})
			setAbsenceSheet(null)
		} catch {
			// Surfaced reactively via `updateAbsence.error`.
		}
	}

	async function handleDeleteAbsence() {
		if (!absenceSheet?.absenceId) return
		try {
			await deleteAbsence.mutateAsync({
				path: {
					organization_id: organizationId,
					absence_id: absenceSheet.absenceId,
				},
			})
			setAbsenceSheet(null)
		} catch {
			// Surfaced reactively via `deleteAbsence.error`.
		}
	}

	const absenceDraft =
		absenceSheet?.draft ?? emptyAbsenceDraft(employeeId, today)
	const absenceErrors = absenceSheet
		? validateAbsenceDraft(absenceDraft, {
				requireEmployee: absenceSheet.mode === 'create',
			})
		: []
	const absenceSaveError =
		createAbsence.error?.message ??
		updateAbsence.error?.message ??
		deleteAbsence.error?.message ??
		null

	return (
		<EmployeeWorkTimeUI
			organizationName={organizationName}
			employee={employee}
			weeklyGap={weeklyGap}
			contractForm={{
				value: contractValue,
				isPending: updateEmployee.isPending,
				error: contractParseError ?? updateEmployee.error?.message ?? null,
				onChange: setContractDraft,
				onSubmit: () => void handleUpdateContract(),
			}}
			rhythmSection={{
				values: rhythmValues,
				otherRhythms,
				openRhythmEffectiveFrom: openRhythm?.effective_from ?? null,
				errors: rhythmErrors,
				isLoading: currentRhythmQuery.isLoading,
				isSaving: replaceRhythm.isPending,
				saveError: rhythmSaveError,
				onEffectiveFromChange: (value) =>
					patchRhythmDraft({ effectiveFrom: value }),
				onEffectiveToChange: (value) =>
					patchRhythmDraft({ effectiveTo: value }),
				onSlotChange: (
					key: string,
					patch: Partial<Pick<RhythmSlotDraft, 'startTime' | 'endTime'>>,
				) =>
					patchRhythmDraft({
						slots: rhythmValues.slots.map((slot) =>
							slot.key === key ? { ...slot, ...patch } : slot,
						),
					}),
				onAddSlot: (weekday) =>
					patchRhythmDraft({
						slots: [...rhythmValues.slots, emptyRhythmSlotDraft(weekday)],
					}),
				onRemoveSlot: (key) =>
					patchRhythmDraft({
						slots: rhythmValues.slots.filter((slot) => slot.key !== key),
					}),
				onSubmit: () => void handleSubmitRhythm(),
			}}
			workSlotsSection={{
				values: workSlotsValues,
				errors: workSlotsErrors,
				isLoading: workTimeQuery.isLoading,
				isSaving: replaceWorkSlots.isPending,
				saveError: replaceWorkSlots.error?.message ?? null,
				onFromChange: (value) => {
					setWorkSlotsRange((range) => ({ ...range, from: value }))
					setWorkSlotsDraft(null)
				},
				onToChange: (value) => {
					setWorkSlotsRange((range) => ({ ...range, to: value }))
					setWorkSlotsDraft(null)
				},
				onSlotChange: (
					key: string,
					patch: Partial<
						Pick<WorkSlotDraft, 'workDate' | 'startTime' | 'endTime'>
					>,
				) =>
					patchWorkSlotsDraft({
						slots: workSlotsValues.slots.map((slot) =>
							slot.key === key ? { ...slot, ...patch } : slot,
						),
					}),
				onAddSlot: () =>
					patchWorkSlotsDraft({
						slots: [
							...workSlotsValues.slots,
							emptyWorkSlotDraft(workSlotsValues.from),
						],
					}),
				onRemoveSlot: (key) =>
					patchWorkSlotsDraft({
						slots: workSlotsValues.slots.filter((slot) => slot.key !== key),
					}),
				onSubmit: () => void handleSubmitWorkSlots(),
			}}
			absencesSection={{
				absences: employeeAbsences,
				isLoading: absencesQuery.isLoading,
				onCreate: handleCreateAbsence,
				onSelect: handleSelectAbsence,
			}}
			absenceSheet={{
				open: absenceSheet !== null,
				mode: absenceSheet?.mode ?? 'create',
				values: absenceDraft,
				employees: employeeOptions,
				errors: absenceErrors,
				isSaving: createAbsence.isPending || updateAbsence.isPending,
				isDeleting: deleteAbsence.isPending,
				saveError: absenceSaveError,
				onChange: handleAbsenceDraftChange,
				onSubmit: () => void handleSubmitAbsence(),
				onDelete:
					absenceSheet?.mode === 'edit'
						? () => void handleDeleteAbsence()
						: undefined,
				onOpenChange: handleAbsenceSheetOpenChange,
			}}
		/>
	)
}

/** Closed/future rhythm versions from either read, deduplicated, excluding the open one. */
function dedupeRhythms(rhythms: Rhythm[], openRhythm: Rhythm | null): Rhythm[] {
	const seen = new Set<string>()
	const result: Rhythm[] = []
	for (const rhythm of rhythms) {
		if (openRhythm && rhythm.id === openRhythm.id) continue
		if (seen.has(rhythm.id)) continue
		seen.add(rhythm.id)
		result.push(rhythm)
	}
	return result
}

function errorStatus(error: unknown): number | undefined {
	if (error && typeof error === 'object' && 'status' in error) {
		const status = (error as { status?: unknown }).status
		return typeof status === 'number' ? status : undefined
	}
	return undefined
}

/**
 * The backend rejects an `effective_from` earlier than the version currently
 * open with a 409 and an English, implementation-facing message (see
 * `WorkTimeService::replace_rhythm`). The form already warns about this
 * proactively (see `validateRhythmDraft`), but a race is still possible, so
 * this turns the raw conflict into the same actionable, French explanation.
 */
function rhythmConflictMessage(openEffectiveFrom: string | null): string {
	return openEffectiveFrom
		? `Impossible de faire démarrer cette version avant celle en cours (${formatDateFr(openEffectiveFrom)}). Choisissez cette date ou une date ultérieure, ou modifiez d’abord la version en cours.`
		: 'Impossible de faire démarrer cette version avant celle actuellement en cours.'
}
