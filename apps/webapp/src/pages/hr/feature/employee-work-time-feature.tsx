import { AlertCircle, UserX } from 'lucide-react'
import { useState } from 'react'
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
	addDaysIso,
	computeWeeklyGap,
	draftToRhythmSlots,
	draftToWorkSlots,
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
import { EmployeeWorkTimeUI } from '#/pages/hr/ui/employee-work-time-ui'

const WORK_SLOTS_WINDOW_DAYS = 13

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
					name: employee.name,
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
