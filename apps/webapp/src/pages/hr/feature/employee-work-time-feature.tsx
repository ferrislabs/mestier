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
	useEmployeeCostBases,
	useReferenceCatalog,
	useSetEmployeeCostBasis,
	useUpsertEmployeeProfile,
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
	browserTimeZone,
	type CostBasisEntry,
	type CostBasisFormValues,
	computeWeeklyGap,
	draftToRhythmSlots,
	draftToWorkSlots,
	emptyCostBasisDraft,
	emptyRhythmSlotDraft,
	emptyWorkSlotDraft,
	eurosToCents,
	findOpenRhythm,
	formatDateFr,
	formatDurationMinutes,
	parseDurationLabel,
	type RhythmFormValues,
	type RhythmSlotDraft,
	rhythmToDraft,
	todayIsoDate,
	validateCostBasisDraft,
	validateRhythmDraft,
	validateWorkSlotsDraft,
	type WorkSlotDraft,
	type WorkSlotsFormValues,
	workSlotsToDraft,
} from '#/pages/hr/types'
import type { AbsenceMemberOption } from '#/pages/hr/ui/absence-form-sheet'
import { EmployeeWorkTimeUI } from '#/pages/hr/ui/employee-work-time-ui'

const WORK_SLOTS_WINDOW_DAYS = 13

interface AbsenceSheetState {
	mode: 'create' | 'edit'
	absenceId: string | null
	draft: AbsenceFormValues
}

interface EmployeeWorkTimeFeatureProps {
	memberId: string
}

export function EmployeeWorkTimeFeature({
	memberId,
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
			key={`${activeOrganization.id}:${memberId}`}
			organizationId={activeOrganization.id}
			memberId={memberId}
		/>
	)
}

interface EmployeeWorkTimeScreenProps {
	organizationId: string
	memberId: string
}

function EmployeeWorkTimeScreen({
	organizationId,
	memberId,
}: EmployeeWorkTimeScreenProps) {
	const catalog = useReferenceCatalog(organizationId, {
		equipment: false,
		serviceRates: false,
		products: false,
	})
	const member =
		catalog.members.data?.data.find((item) => item.id === memberId) ?? null
	const profile =
		catalog.employeeProfiles.data?.data.find(
			(item) => item.member_id === memberId,
		) ?? null

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
	const currentRhythmQuery = useWorkTime(memberId, todayRange)
	const workTimeQuery = useWorkTime(memberId, workSlotsRange)

	const upsertProfile = useUpsertEmployeeProfile()
	const replaceRhythm = useReplaceRhythm(memberId)
	const replaceWorkSlots = useReplaceWorkSlots(memberId)
	const costBasesQuery = useEmployeeCostBases(
		profile?.id ?? '',
		Boolean(profile),
	)
	const setCostBasis = useSetEmployeeCostBasis()

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

	const [costBasisDraft, setCostBasisDraft] =
		useState<CostBasisFormValues | null>(null)
	const costBasisValues =
		costBasisDraft ??
		emptyCostBasisDraft(today, {
			isSalaried: profile?.is_salaried ?? false,
			hourlyRateCents: profile?.hourly_rate_cents ?? null,
			monthlyCostCents: profile?.monthly_cost_cents ?? null,
		})

	const absencesQuery = useAbsences(organizationId)
	const createAbsence = useCreateAbsence()
	const updateAbsence = useUpdateAbsence()
	const deleteAbsence = useDeleteAbsence()
	const [absenceSheet, setAbsenceSheet] = useState<AbsenceSheetState | null>(
		null,
	)

	if (catalog.members.isLoading || catalog.employeeProfiles.isLoading) {
		return <EmployeeWorkTimeUI.Loading />
	}

	if (!member) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<UserX className="size-6 text-muted-foreground" />
				</div>
				<div>
					<p className="font-semibold">Personne introuvable</p>
					<p className="text-sm text-muted-foreground">
						Aucune personne ne correspond à cet identifiant dans cette
						organisation.
					</p>
				</div>
			</div>
		)
	}

	const contractValue =
		contractDraft ??
		formatDurationMinutes(profile?.weekly_contract_minutes ?? 0)
	const contractMinutesForGap =
		parseDurationLabel(contractValue) ?? profile?.weekly_contract_minutes ?? 0
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

	const costBasisErrors = validateCostBasisDraft(costBasisValues)
	const costHistory: CostBasisEntry[] = (costBasesQuery.data?.data ?? []).map(
		(entry) => ({
			id: entry.id,
			effectiveFrom: entry.effective_from,
			effectiveTo: entry.effective_to ?? null,
			isSalaried: entry.is_salaried,
			hourlyRateCents: entry.hourly_rate_cents ?? null,
			monthlyCostCents: entry.monthly_cost_cents ?? null,
			effectiveHourlyRateCents: entry.effective_hourly_rate_cents ?? null,
		}),
	)
	const openCostBasis =
		costHistory.find((entry) => entry.effectiveTo === null) ?? null

	const handleSubmitCostBasis = async () => {
		if (!profile || costBasisErrors.length > 0) return
		try {
			await setCostBasis.mutateAsync({
				path: { employee_id: profile.id },
				body: {
					effective_from: costBasisValues.effectiveFrom,
					is_salaried: costBasisValues.isSalaried,
					hourly_rate_cents: eurosToCents(costBasisValues.hourlyRate),
					monthly_cost_cents: costBasisValues.isSalaried
						? eurosToCents(costBasisValues.monthlyCost)
						: null,
					weekly_contract_minutes: profile.weekly_contract_minutes,
				},
			})
			setCostBasisDraft(null)
		} catch {
			// Surfaced reactively via `setCostBasis.error` — the draft is kept so
			// the user doesn't lose what they typed.
		}
	}

	const handleUpdateContract = async () => {
		const minutes = parseDurationLabel(contractValue)
		if (minutes === null) return
		try {
			// The whole profile, not just the contract: this is a `PUT`, so every
			// field left out is a field cleared. Omitting `is_salaried` used to
			// move a salaried person back onto an hourly basis and wipe the
			// monthly amount they had just entered.
			await upsertProfile.mutateAsync({
				path: { member_id: memberId },
				body: {
					hourly_rate_cents: profile?.hourly_rate_cents ?? null,
					is_salaried: profile?.is_salaried ?? false,
					monthly_cost_cents: profile?.monthly_cost_cents ?? null,
					weekly_contract_minutes: minutes,
				},
			})
			setContractDraft(null)
		} catch {
			// Surfaced reactively via `upsertProfile.error` — the draft is kept
			// so the user doesn't lose what they typed.
		}
	}

	const handleSubmitRhythm = async () => {
		if (rhythmErrors.length > 0) return
		try {
			await replaceRhythm.mutateAsync({
				path: { member_id: memberId },
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
				path: { member_id: memberId },
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

	const memberOptions: AbsenceMemberOption[] = (
		catalog.members.data?.data ?? []
	).map((candidate) => ({
		memberId: candidate.id,
		displayName: candidate.display_name,
	}))

	const memberAbsences: AbsenceListItem[] = (absencesQuery.data?.data ?? [])
		.filter((absence) => absence.member_id === memberId)
		.map((absence) => ({
			id: absence.id,
			...absenceToDraft({ ...absence, absence_kind: absence.kind }, timeZone),
		}))
		.sort((a, b) => a.range.from.localeCompare(b.range.from))

	function handleCreateAbsence() {
		setAbsenceSheet({
			mode: 'create',
			absenceId: null,
			draft: emptyAbsenceDraft(memberId, today),
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

	const absenceDraft = absenceSheet?.draft ?? emptyAbsenceDraft(memberId, today)
	const absenceErrors = absenceSheet
		? validateAbsenceDraft(absenceDraft, {
				requireMember: absenceSheet.mode === 'create',
			})
		: []
	const absenceSaveError =
		createAbsence.error?.message ??
		updateAbsence.error?.message ??
		deleteAbsence.error?.message ??
		null

	return (
		<EmployeeWorkTimeUI
			member={member}
			hourlyRateCents={profile?.hourly_rate_cents ?? null}
			isSalaried={profile?.is_salaried ?? false}
			monthlyCostCents={profile?.monthly_cost_cents ?? null}
			effectiveHourlyRateCents={profile?.effective_hourly_rate_cents ?? null}
			openCostBasisEffectiveFrom={openCostBasis?.effectiveFrom ?? null}
			weeklyGap={weeklyGap}
			contractForm={{
				value: contractValue,
				isPending: upsertProfile.isPending,
				error: contractParseError ?? upsertProfile.error?.message ?? null,
				onChange: setContractDraft,
				onSubmit: () => void handleUpdateContract(),
			}}
			costHistorySection={{
				history: costHistory,
				isLoading: costBasesQuery.isLoading,
				form: {
					values: costBasisValues,
					errors: costBasisErrors,
					isSaving: setCostBasis.isPending,
					saveError: setCostBasis.error?.message ?? null,
					onChange: (patch) =>
						setCostBasisDraft({ ...costBasisValues, ...patch }),
					onSubmit: () => void handleSubmitCostBasis(),
				},
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
				absences: memberAbsences,
				isLoading: absencesQuery.isLoading,
				onCreate: handleCreateAbsence,
				onSelect: handleSelectAbsence,
			}}
			absenceSheet={{
				open: absenceSheet !== null,
				mode: absenceSheet?.mode ?? 'create',
				values: absenceDraft,
				members: memberOptions,
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
