export type IsoDate = string

export type Weekday = 0 | 1 | 2 | 3 | 4 | 5 | 6

export type LunchBreak = { start: string; end: string }

export type WorkDay = {
	start: string
	end: string
	lunch?: LunchBreak | null
}

export type WeekSchedule = readonly (WorkDay | null)[]

export type ContractType =
	| 'cdi'
	| 'cdd'
	| 'apprentissage'
	| 'interim'
	| 'gerant'

export type MemberStatus = 'active' | 'archived'

export type DocumentKind =
	| 'contract'
	| 'identity'
	| 'diploma'
	| 'medical'
	| 'other'

export type MemberDocument = {
	id: string
	name: string
	kind: DocumentKind
	addedAt: IsoDate
	fileKey: string
}

export type ScheduleUnit = 'weeks' | 'months'

export type ScheduleBlock = {
	length: number
	unit: ScheduleUnit
	label: string
	week: WeekSchedule
}

export type ContractSchedule =
	| { kind: 'weekly'; week: WeekSchedule }
	| { kind: 'rotation'; anchor: IsoDate; blocks: readonly ScheduleBlock[] }
	| { kind: 'variable'; defaultDay: WorkDay }

export type Contract = {
	id: string
	type: ContractType
	startDate: IsoDate
	endDate: IsoDate | null
	trialEndDate: IsoDate | null
	schedule: ContractSchedule
}

export type ContractDraft = Omit<Contract, 'id'>

export type PostalAddress = {
	street: string
	complement: string
	postalCode: string
	city: string
	country: string
}

export type Member = {
	id: string
	firstName: string
	lastName: string
	workEmail: string
	personalEmail: string
	mobile: string
	landline: string
	address: PostalAddress
	jobTitle: string
	hue: number
	contracts: readonly Contract[]
	workPermitEndDate: IsoDate | null
	managerId: string | null
	status: MemberStatus
	archivedAt: IsoDate | null
	documents: readonly MemberDocument[]
}

export type MemberDraft = Omit<
	Member,
	'id' | 'status' | 'archivedAt' | 'documents' | 'hue' | 'contracts'
> & { contract: ContractDraft }

export type CountMode = 'working' | 'workable' | 'calendar'

export type AbsenceKind = 'leave' | 'sick' | 'unavailable'

export type AbsenceType = {
	id: string
	name: string
	hue: number
	kind: AbsenceKind
	countMode: CountMode
	accountId: string | null
	paid: boolean
	needsApproval: boolean
	needsProof: boolean
	archived: boolean
}

export type AbsenceTypeDraft = Omit<AbsenceType, 'id' | 'archived'>

export type DayPart = 'morning' | 'afternoon'

export type AbsenceStatus = 'pending' | 'approved' | 'rejected' | 'cancelled'

export type Absence = {
	id: string
	memberId: string
	typeId: string
	startDate: IsoDate
	startPart: DayPart
	endDate: IsoDate
	endPart: DayPart
	comment: string
	proofKey: string | null
	status: AbsenceStatus
	createdAt: string
	createdById: string
	decidedById: string | null
	decidedAt: string | null
	decisionComment: string
}

export type AbsenceSpanInput = Pick<
	Absence,
	'memberId' | 'typeId' | 'startDate' | 'startPart' | 'endDate' | 'endPart'
>

export type AbsenceDraft = Pick<
	Absence,
	| 'memberId'
	| 'typeId'
	| 'startDate'
	| 'startPart'
	| 'endDate'
	| 'endPart'
	| 'comment'
> & { proofName: string | null }

export type Acquisition =
	| { kind: 'monthly'; perMonth: number }
	| { kind: 'annual'; days: number }

export type LeaveAccount = {
	id: string
	name: string
	acquisition: Acquisition
	periodStartMonth: number
	carryOverMax: number
}

export type AdjustmentKind = 'adjustment' | 'carry_over'

export type BalanceAdjustment = {
	id: string
	memberId: string
	accountId: string
	date: IsoDate
	days: number
	kind: AdjustmentKind
	reason: string
	authorId: string
}

export type AdjustmentDraft = Pick<
	BalanceAdjustment,
	'memberId' | 'accountId' | 'date' | 'days' | 'kind' | 'reason'
>

export type DayOverrideKind = 'work' | 'off'

export type DayOverride = {
	memberId: string
	date: IsoDate
	kind: DayOverrideKind
	start: string
	end: string
	lunch?: LunchBreak | null
	note: string
}

export type SurchargeKind =
	| 'overtime'
	| 'sunday'
	| 'holiday'
	| 'night'
	| 'other'

export type Surcharge = {
	id: string
	name: string
	rate: number
	kind: SurchargeKind
	archived: boolean
}

export type SurchargeDraft = Omit<Surcharge, 'id' | 'archived'>

export type ExtraStatus = AbsenceStatus

export type ExtraPeriod = {
	id: string
	memberId: string
	date: IsoDate
	start: string
	end: string
	lunch: LunchBreak | null
	reason: string
	surchargeId: string | null
	rate: number
	status: ExtraStatus
	createdAt: string
	createdById: string
	decidedById: string | null
	decidedAt: string | null
	decisionComment: string
}

export type ExtraDraft = Pick<
	ExtraPeriod,
	| 'memberId'
	| 'date'
	| 'start'
	| 'end'
	| 'lunch'
	| 'reason'
	| 'surchargeId'
	| 'rate'
>

export type RhState = {
	members: readonly Member[]
	absenceTypes: readonly AbsenceType[]
	accounts: readonly LeaveAccount[]
	absences: readonly Absence[]
	adjustments: readonly BalanceAdjustment[]
	overrides: readonly DayOverride[]
	surcharges: readonly Surcharge[]
	extras: readonly ExtraPeriod[]
}

export type ActionContext = {
	now: string
	today: IsoDate
	actorId: string
	newId: () => string
}
