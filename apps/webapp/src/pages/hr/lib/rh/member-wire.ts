import type { Schemas } from '#/api/api.client'
import { weekly } from '#/pages/hr/lib/rh/contract-schedule'
import type {
	Contract,
	ContractDraft,
	ContractSchedule,
	ContractType,
	DocumentKind,
	LunchBreak,
	Member,
	MemberDocument,
	RhState,
	ScheduleBlock,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

export type ContractRecord = Schemas.MemberContractResponse
export type ContractRequest = Schemas.PutMemberContractRequest
export type DocumentRecord = Schemas.MemberDocumentResponse
export type DocumentRequest = Schemas.AddMemberDocumentRequest

type WireContractType = Schemas.ContractType
type WireDocumentKind = Schemas.DocumentKind
type WireSchedule = Schemas.ScheduleInput
type WireWorkDay = Schemas.WorkDayInput
type WireBlock = Schemas.BlockInput

export const CONTRACT_TYPE_TO_WIRE: Record<ContractType, WireContractType> = {
	cdi: 'CDI',
	cdd: 'CDD',
	apprentissage: 'APPRENTICESHIP',
	interim: 'INTERIM',
	gerant: 'MANAGER',
}

export const CONTRACT_TYPE_FROM_WIRE: Record<WireContractType, ContractType> = {
	CDI: 'cdi',
	CDD: 'cdd',
	APPRENTICESHIP: 'apprentissage',
	INTERIM: 'interim',
	MANAGER: 'gerant',
}

export const DOCUMENT_KIND_TO_WIRE: Record<DocumentKind, WireDocumentKind> = {
	contract: 'CONTRACT',
	identity: 'IDENTITY',
	diploma: 'DIPLOMA',
	medical: 'MEDICAL',
	other: 'OTHER',
}

export const DOCUMENT_KIND_FROM_WIRE: Record<WireDocumentKind, DocumentKind> = {
	CONTRACT: 'contract',
	IDENTITY: 'identity',
	DIPLOMA: 'diploma',
	MEDICAL: 'medical',
	OTHER: 'other',
}

function workDayOf(day: WireWorkDay): WorkDay {
	return {
		start: day.start,
		end: day.end,
		lunch: day.lunch ? { start: day.lunch.start, end: day.lunch.end } : null,
	}
}

function lunchRequestOf(
	lunch: LunchBreak | null | undefined,
): WireWorkDay['lunch'] {
	return lunch ? { start: lunch.start, end: lunch.end } : null
}

function workDayRequestOf(day: WorkDay): WireWorkDay {
	return { start: day.start, end: day.end, lunch: lunchRequestOf(day.lunch) }
}

function blockOf(block: WireBlock): ScheduleBlock {
	return {
		length: block.length,
		unit: block.unit,
		label: block.label ?? '',
		week: block.week.map((day) => (day ? workDayOf(day) : null)),
	}
}

function blockRequestOf(block: ScheduleBlock): WireBlock {
	return {
		length: block.length,
		unit: block.unit,
		label: block.label,
		week: block.week.map((day) => (day ? workDayRequestOf(day) : null)),
	}
}

export function scheduleOf(schedule: WireSchedule): ContractSchedule {
	switch (schedule.kind) {
		case 'weekly':
			return weekly(schedule.week.map((day) => (day ? workDayOf(day) : null)))
		case 'rotation':
			return {
				kind: 'rotation',
				anchor: schedule.anchor,
				blocks: schedule.blocks.map(blockOf),
			}
		case 'variable':
			return { kind: 'variable', defaultDay: workDayOf(schedule.default_day) }
	}
}

export function scheduleRequestOf(schedule: ContractSchedule): WireSchedule {
	switch (schedule.kind) {
		case 'weekly':
			return {
				kind: 'weekly',
				week: schedule.week.map((day) => (day ? workDayRequestOf(day) : null)),
			}
		case 'rotation':
			return {
				kind: 'rotation',
				anchor: schedule.anchor,
				blocks: schedule.blocks.map(blockRequestOf),
			}
		case 'variable':
			return {
				kind: 'variable',
				default_day: workDayRequestOf(schedule.defaultDay),
			}
	}
}

export function contractOf(record: ContractRecord): Contract {
	return {
		id: record.id,
		type: CONTRACT_TYPE_FROM_WIRE[record.type],
		startDate: record.start_date,
		endDate: record.end_date ?? null,
		trialEndDate: record.trial_end_date ?? null,
		schedule: scheduleOf(record.schedule),
	}
}

export function contractRequestOf(draft: ContractDraft): ContractRequest {
	return {
		type: CONTRACT_TYPE_TO_WIRE[draft.type],
		start_date: draft.startDate,
		end_date: draft.endDate,
		trial_end_date: draft.trialEndDate,
		schedule: scheduleRequestOf(draft.schedule),
	}
}

export function documentOf(record: DocumentRecord): MemberDocument {
	return {
		id: record.id,
		name: record.name,
		kind: DOCUMENT_KIND_FROM_WIRE[record.kind],
		addedAt: record.added_at.slice(0, 10),
		fileKey: record.file_key,
	}
}

export function documentRequestOf(
	name: string,
	kind: DocumentKind,
	fileKey: string,
): DocumentRequest {
	return {
		name: name.trim(),
		kind: DOCUMENT_KIND_TO_WIRE[kind],
		file_key: fileKey,
	}
}

function groupByMember<Record extends { member_id: string }, Item>(
	records: readonly Record[],
	convert: (record: Record) => Item,
): Map<string, Item[]> {
	const grouped = new Map<string, Item[]>()
	for (const record of records) {
		const items = grouped.get(record.member_id) ?? []
		items.push(convert(record))
		grouped.set(record.member_id, items)
	}
	return grouped
}

function bySortedStart(contracts: Contract[]): Contract[] {
	return contracts.sort((a, b) => a.startDate.localeCompare(b.startDate))
}

export function overlayContracts(
	state: RhState,
	records: readonly ContractRecord[],
): RhState {
	const byMember = groupByMember(records, contractOf)
	return {
		...state,
		members: state.members.map(
			(member): Member => ({
				...member,
				contracts: bySortedStart(byMember.get(member.id) ?? []),
			}),
		),
	}
}

export function overlayDocuments(
	state: RhState,
	records: readonly DocumentRecord[],
): RhState {
	const byMember = groupByMember(records, documentOf)
	return {
		...state,
		members: state.members.map(
			(member): Member => ({
				...member,
				documents: (byMember.get(member.id) ?? []).sort((a, b) =>
					a.addedAt.localeCompare(b.addedAt),
				),
			}),
		),
	}
}

export type ContractFailureField =
	| 'startDate'
	| 'endDate'
	| 'trialEndDate'
	| 'schedule'

export type ContractFailure = {
	field: ContractFailureField | null
	overlap: boolean
	message: string
}

const FIELD_OF_PATH: Record<string, ContractFailureField> = {
	start_date: 'startDate',
	end_date: 'endDate',
	trial_end_date: 'trialEndDate',
}

const WEEKDAY_NAMES = [
	'lundi',
	'mardi',
	'mercredi',
	'jeudi',
	'vendredi',
	'samedi',
	'dimanche',
]

const OVERLAP_CODE = 'E_MEMBER_CONTRACT_OVERLAP'
const OVERLAP_MESSAGE = 'Ce contrat chevauche un autre contrat.'

type ApiFailure = {
	status?: number
	message?: string
	data?: { code?: string; message?: string; details?: { field?: string } }
}

function weekdayLabel(index: number): string {
	return WEEKDAY_NAMES[index] ?? `jour ${index + 1}`
}

export function schedulePathLabel(path: string): string | null {
	const block =
		/^schedule\.blocks\[(\d+)\](?:\.(length|week)(?:\[(\d+)\])?)?$/.exec(path)
	if (block) {
		const sequence = `Séquence ${Number(block[1]) + 1}`
		return block[3] === undefined
			? sequence
			: `${sequence}, ${weekdayLabel(Number(block[3]))}`
	}
	const day = /^schedule\.week\[(\d+)\]$/.exec(path)
	if (day) return `Rythme, ${weekdayLabel(Number(day[1]))}`
	if (path === 'schedule.default_day') return 'Horaires par défaut'
	if (path.startsWith('schedule')) return 'Rythme'
	return null
}

export function contractFailureOf(error: unknown): ContractFailure {
	const failure = (
		typeof error === 'object' && error !== null ? error : {}
	) as ApiFailure
	const message =
		failure.data?.message ?? failure.message ?? 'Une erreur est survenue.'
	if (failure.data?.code === OVERLAP_CODE || failure.status === 409)
		return { field: 'startDate', overlap: true, message: OVERLAP_MESSAGE }
	const path = failure.data?.details?.field ?? ''
	const known = FIELD_OF_PATH[path]
	if (known) return { field: known, overlap: false, message }
	const label = schedulePathLabel(path)
	if (label)
		return {
			field: 'schedule',
			overlap: false,
			message: `${label} : ${message}`,
		}
	return { field: null, overlap: false, message }
}
