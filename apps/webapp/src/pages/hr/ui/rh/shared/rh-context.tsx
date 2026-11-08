import { createContext, type ReactNode, useContext } from 'react'
import { buildOrgPath } from '#/modules/org-path'
import type { DayPlan } from '#/pages/hr/lib/rh/day-planning'
import type { ProfileFields } from '#/pages/hr/lib/rh/member-profile'
import type { AccountSettingsDraft } from '#/pages/hr/lib/rh/rights-model'
import type {
	AbsenceDraft,
	AbsenceTypeDraft,
	ActionContext,
	AdjustmentDraft,
	ContractDraft,
	DayOverride,
	DocumentKind,
	ExtraDraft,
	IsoDate,
	LeaveAccount,
	RhState,
	SurchargeDraft,
} from '#/pages/hr/lib/rh/types'

export type RhUpdate = (
	update: (current: RhState, context: ActionContext) => RhState,
) => Promise<void>

export interface RhProfileActions {
	save: (memberId: string, fields: ProfileFields) => Promise<void>
	archive: (memberId: string) => Promise<void>
	restore: (memberId: string) => Promise<void>
}

export interface RhContractActions {
	add: (memberId: string, draft: ContractDraft) => Promise<void>
	update: (
		memberId: string,
		contractId: string,
		draft: ContractDraft,
	) => Promise<void>
	remove: (memberId: string, contractId: string) => Promise<void>
}

export interface RhDocumentActions {
	add: (memberId: string, file: File, kind: DocumentKind) => Promise<void>
	remove: (memberId: string, documentId: string) => Promise<void>
}

export interface RhAbsenceActions {
	create: (draft: AbsenceDraft, proof: File | null) => Promise<void>
	approve: (absenceId: string) => Promise<void>
	reject: (absenceId: string, comment: string) => Promise<void>
	cancel: (absenceId: string) => Promise<void>
}

export interface RhAbsenceTypeActions {
	save: (typeId: string | null, draft: AbsenceTypeDraft) => Promise<void>
	archive: (typeId: string) => Promise<void>
	restore: (typeId: string) => Promise<void>
}

export interface RhLeaveActions {
	saveAccount: (
		account: LeaveAccount,
		settings: AccountSettingsDraft,
	) => Promise<void>
	archiveAccount: (accountId: string) => Promise<void>
	restoreAccount: (accountId: string) => Promise<void>
	addAdjustment: (draft: AdjustmentDraft) => Promise<void>
	removeAdjustment: (adjustmentId: string) => Promise<void>
}

export interface RhExtraActions {
	create: (draft: ExtraDraft, approve: boolean) => Promise<void>
	approve: (extraId: string) => Promise<void>
	reject: (extraId: string, comment: string) => Promise<void>
	cancel: (extraId: string) => Promise<void>
}

export interface RhSurchargeActions {
	save: (surchargeId: string | null, draft: SurchargeDraft) => Promise<void>
	archive: (surchargeId: string) => Promise<void>
	restore: (surchargeId: string) => Promise<void>
}

export interface RhOverrideActions {
	put: (draft: DayOverride) => Promise<void>
	clear: (memberId: string, date: IsoDate) => Promise<void>
	plan: (plan: DayPlan) => Promise<void>
	clearRange: (
		memberId: string,
		startDate: IsoDate,
		endDate: IsoDate,
	) => Promise<void>
}

export interface RhContextValue {
	state: RhState
	update: RhUpdate
	profiles: RhProfileActions
	contracts: RhContractActions
	documents: RhDocumentActions
	absences: RhAbsenceActions
	absenceTypes: RhAbsenceTypeActions
	leave: RhLeaveActions
	extras: RhExtraActions
	surcharges: RhSurchargeActions
	overrides: RhOverrideActions
	actorId: string
	organizationId: string
	organizationSlug: string
}

const RhContext = createContext<RhContextValue | null>(null)

export function RhProvider({
	value,
	children,
}: {
	value: RhContextValue
	children: ReactNode
}) {
	return <RhContext.Provider value={value}>{children}</RhContext.Provider>
}

function useRhContext(): RhContextValue {
	const value = useContext(RhContext)
	if (!value) throw new Error('RhProvider manquant')
	return value
}

export function useRhState(): RhState {
	return useRhContext().state
}

export function useRhUpdate(): RhUpdate {
	return useRhContext().update
}

export function useRhProfileActions(): RhProfileActions {
	return useRhContext().profiles
}

export function useRhContractActions(): RhContractActions {
	return useRhContext().contracts
}

export function useRhDocumentActions(): RhDocumentActions {
	return useRhContext().documents
}

export function useRhAbsenceActions(): RhAbsenceActions {
	return useRhContext().absences
}

export function useRhAbsenceTypeActions(): RhAbsenceTypeActions {
	return useRhContext().absenceTypes
}

export function useRhLeaveActions(): RhLeaveActions {
	return useRhContext().leave
}

export function useRhExtraActions(): RhExtraActions {
	return useRhContext().extras
}

export function useRhSurchargeActions(): RhSurchargeActions {
	return useRhContext().surcharges
}

export function useRhOverrideActions(): RhOverrideActions {
	return useRhContext().overrides
}

export function useRhOrganizationId(): string {
	return useRhContext().organizationId
}

export function useRhActorId(): string {
	return useRhContext().actorId
}

export function useRhOrgPath(): (path: string) => string {
	const { organizationSlug } = useRhContext()
	return (path) => buildOrgPath(organizationSlug, path)
}

export function useRhOrgSlug(): string {
	return useRhContext().organizationSlug
}
