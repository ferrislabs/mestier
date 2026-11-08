type Declaration = { source: string; uses: readonly string[] }

type PayloadContract = { fields: readonly string[]; uses: readonly string[] }

function union(values: readonly string[]): string {
	return values.map((value) => `'${value}'`).join(' | ')
}

function block(keyword: string, name: string, fields: readonly string[]) {
	return `${keyword} ${name} {\n${fields.map((field) => `  ${field}`).join('\n')}\n}`
}

const QUOTE_STATUSES = ['DRAFT', 'SENT', 'ACCEPTED', 'DECLINED', 'CANCELLED']
const INVOICE_STATUSES = [
	'DRAFT',
	'ISSUED',
	'PAID',
	'PARTIALLY_PAID',
	'CANCELLED',
]
const SUPPLIER_INVOICE_STATUSES = ['RECEIVED', 'CONFIRMED', 'REJECTED']
const CUSTOMER_STATUSES = ['PROSPECT', 'CLIENT', 'ARCHIVED']
const LEAD_SOURCES = [
	'bouche_a_oreille',
	'site_web',
	'recommandation',
	'salon',
	'architecte',
	'autre',
]
const DEAL_STAGES = ['NEW', 'VISIT', 'QUOTE_SENT', 'NEGOTIATION', 'WON', 'LOST']
const DECISION_STATUSES = ['PENDING', 'APPROVED', 'REJECTED', 'CANCELLED']
const ABSENCE_KINDS = ['LEAVE', 'SICK', 'UNAVAILABLE']
const DAY_PARTS = ['MORNING', 'AFTERNOON']

const DECLARATIONS: Record<string, Declaration> = {
	Uuid: { source: 'type Uuid = string', uses: [] },
	Cents: { source: 'type Cents = number', uses: [] },
	DateTime: { source: 'type DateTime = string', uses: [] },
	IsoDate: { source: 'type IsoDate = string', uses: [] },
	VatBreakdownLine: {
		source: block('interface', 'VatBreakdownLine', [
			'rate_bp: number',
			'vat_cents: Cents',
		]),
		uses: ['Cents'],
	},
	QuoteStatus: {
		source: `type QuoteStatus = ${union(QUOTE_STATUSES)}`,
		uses: [],
	},
	Quote: {
		source: block('interface', 'Quote', [
			'id: Uuid',
			'organization_id: Uuid',
			'reference: string | null',
			'title: string',
			'customer_id: Uuid',
			'customer_context_id: Uuid',
			'status: QuoteStatus',
			'net_cents: Cents',
			'vat_breakdown: VatBreakdownLine[]',
			'gross_cents: Cents',
			'created_at: DateTime',
			'updated_at: DateTime',
		]),
		uses: ['QuoteStatus', 'VatBreakdownLine', 'Uuid', 'Cents', 'DateTime'],
	},
	InvoiceStatus: {
		source: `type InvoiceStatus = ${union(INVOICE_STATUSES)}`,
		uses: [],
	},
	InvoiceKind: {
		source: `type InvoiceKind = ${union(['STANDARD', 'DEPOSIT', 'FINAL', 'CREDIT_NOTE'])}`,
		uses: [],
	},
	Invoice: {
		source: block('interface', 'Invoice', [
			'id: Uuid',
			'organization_id: Uuid',
			'number: string | null',
			'kind: InvoiceKind',
			'project_id: Uuid | null',
			'customer_id: Uuid',
			'customer_context_id: Uuid',
			'status: InvoiceStatus',
			'issued_at: DateTime | null',
			'due_at: DateTime | null',
			'net_cents: Cents',
			'vat_breakdown: VatBreakdownLine[]',
			'gross_cents: Cents',
			'created_at: DateTime',
			'updated_at: DateTime',
		]),
		uses: [
			'InvoiceKind',
			'InvoiceStatus',
			'VatBreakdownLine',
			'Uuid',
			'Cents',
			'DateTime',
		],
	},
	InvoicePayment: {
		source: block('interface', 'InvoicePayment', [
			'id: Uuid',
			'invoice_id: Uuid',
			'amount_cents: Cents',
			'paid_on: IsoDate',
			'method: string',
			'reference: string | null',
			'note: string | null',
			'recorded_by: Uuid',
		]),
		uses: ['Uuid', 'Cents', 'IsoDate'],
	},
	TicketStatus: {
		source: `type TicketStatus = ${union(['new', 'in_progress', 'waiting_customer', 'resolved'])}`,
		uses: [],
	},
	TicketPriority: {
		source: `type TicketPriority = ${union(['low', 'normal', 'high', 'urgent'])}`,
		uses: [],
	},
	Badge: {
		source: block('interface', 'Badge', ['label: string', 'tone: string']),
		uses: [],
	},
	Ticket: {
		source: block('interface', 'Ticket', [
			'id: Uuid',
			'organization_id: Uuid',
			'number: number',
			'subject: string',
			'body: string',
			'status: TicketStatus',
			'priority: TicketPriority',
			'assignee_member_id: Uuid | null',
			'assigned_at: DateTime | null',
			'custom_fields: Record<string, string>',
			'badges: Badge[]',
			'external_id: string | null',
			'created_at: DateTime',
			'updated_at: DateTime',
		]),
		uses: ['TicketStatus', 'TicketPriority', 'Badge', 'Uuid', 'DateTime'],
	},
	InterventionStatus: {
		source: `type InterventionStatus = ${union(['PROPOSED', 'VALIDATED', 'REJECTED'])}`,
		uses: [],
	},
	MaintenanceIntervention: {
		source: block('interface', 'MaintenanceIntervention', [
			'id: Uuid',
			'organization_id: Uuid',
			'line_id: Uuid',
			'done_on: IsoDate',
			'status: InterventionStatus',
			'task_id: Uuid',
			'proposed_by: Uuid',
			'resolved_by: Uuid | null',
			'resolved_at: DateTime | null',
		]),
		uses: ['InterventionStatus', 'Uuid', 'IsoDate', 'DateTime'],
	},
	TimeEntry: {
		source: block('interface', 'TimeEntry', [
			'id: Uuid',
			'organization_id: Uuid',
			'task_id: Uuid',
			'employee_id: Uuid',
			'started_at: DateTime',
			'ended_at: DateTime | null',
		]),
		uses: ['Uuid', 'DateTime'],
	},
	DayLog: {
		source: block('interface', 'DayLog', [
			'id: Uuid',
			'organization_id: Uuid',
			'employee_id: Uuid',
			'work_date: IsoDate',
			'ended_at: DateTime',
		]),
		uses: ['Uuid', 'IsoDate', 'DateTime'],
	},
	AssignmentReportResolution: {
		source: `type AssignmentReportResolution = ${union(['PENDING', 'APPLIED', 'DISMISSED'])}`,
		uses: [],
	},
	AssignmentReport: {
		source: block('interface', 'AssignmentReport', [
			'id: Uuid',
			'organization_id: Uuid',
			'task_assignment_id: Uuid',
			'reported_minutes: number',
			'reported_by: Uuid',
			'resolution: AssignmentReportResolution',
			'resolved_by: Uuid | null',
			'resolved_at: DateTime | null',
		]),
		uses: ['AssignmentReportResolution', 'Uuid', 'DateTime'],
	},
	SupplierInvoiceStatus: {
		source: `type SupplierInvoiceStatus = ${union(SUPPLIER_INVOICE_STATUSES)}`,
		uses: [],
	},
	SupplierInvoiceSource: {
		source: `type SupplierInvoiceSource = ${union(['MANUAL', 'FACTUR_X'])}`,
		uses: [],
	},
	SupplierInvoice: {
		source: block('interface', 'SupplierInvoice', [
			'id: Uuid',
			'organization_id: Uuid',
			'supplier_id: Uuid | null',
			'supplier_name: string',
			'number: string',
			'issued_on: IsoDate',
			'due_on: IsoDate | null',
			'received_at: DateTime',
			'source: SupplierInvoiceSource',
			'status: SupplierInvoiceStatus',
			'currency: string',
			'net_cents: Cents',
			'vat_breakdown: VatBreakdownLine[]',
			'gross_cents: Cents',
		]),
		uses: [
			'SupplierInvoiceSource',
			'SupplierInvoiceStatus',
			'VatBreakdownLine',
			'Uuid',
			'IsoDate',
			'DateTime',
			'Cents',
		],
	},
	CustomerStatus: {
		source: `type CustomerStatus = ${union(CUSTOMER_STATUSES)}`,
		uses: [],
	},
	LeadSource: {
		source: `type LeadSource = ${union(LEAD_SOURCES)}`,
		uses: [],
	},
	Customer: {
		source: block('interface', 'Customer', [
			'id: Uuid',
			'organization_id: Uuid',
			'status: CustomerStatus',
			"kind: 'INDIVIDUAL' | 'PROFESSIONAL'",
			'name: string',
			'email: string | null',
			'phone: string | null',
			'registration_number: string | null',
			'vat_number: string | null',
			'owner_member_id: Uuid | null',
			'lead_source: LeadSource | null',
			'created_at: DateTime',
		]),
		uses: ['CustomerStatus', 'LeadSource', 'Uuid', 'DateTime'],
	},
	DealStage: {
		source: `type DealStage = ${union(DEAL_STAGES)}`,
		uses: [],
	},
	Deal: {
		source: block('interface', 'Deal', [
			'id: Uuid',
			'organization_id: Uuid',
			'customer_id: Uuid',
			'title: string',
			'amount_cents: Cents',
			'stage: DealStage',
			'lost_reason: string | null',
			'expected_close_date: IsoDate | null',
			'owner_member_id: Uuid | null',
			'created_at: DateTime',
			'stage_changed_at: DateTime',
			'closed_at: DateTime | null',
		]),
		uses: ['DealStage', 'Uuid', 'Cents', 'IsoDate', 'DateTime'],
	},
	DecisionStatus: {
		source: `type DecisionStatus = ${union(DECISION_STATUSES)}`,
		uses: [],
	},
	AbsenceKind: {
		source: `type AbsenceKind = ${union(ABSENCE_KINDS)}`,
		uses: [],
	},
	DayPart: {
		source: `type DayPart = ${union(DAY_PARTS)}`,
		uses: [],
	},
	Absence: {
		source: block('interface', 'Absence', [
			'id: Uuid',
			'organization_id: Uuid',
			'member_id: Uuid',
			'kind: AbsenceKind',
			'type_id: Uuid',
			'starts_at: DateTime',
			'ends_at: DateTime',
			'all_day: boolean',
			'start_date: IsoDate',
			'start_part: DayPart',
			'end_date: IsoDate',
			'end_part: DayPart',
			'status: DecisionStatus',
			'note: string | null',
			'proof_file_key: string | null',
			'created_by_member_id: Uuid | null',
			'decided_by_member_id: Uuid | null',
			'decided_at: DateTime | null',
			'decision_comment: string | null',
			'created_at: DateTime',
			'updated_at: DateTime',
		]),
		uses: [
			'AbsenceKind',
			'DayPart',
			'DecisionStatus',
			'Uuid',
			'IsoDate',
			'DateTime',
		],
	},
	ExtraPeriod: {
		source: block('interface', 'ExtraPeriod', [
			'id: Uuid',
			'organization_id: Uuid',
			'member_id: Uuid',
			'date: IsoDate',
			'start: string',
			'end: string',
			'lunch: { start: string; end: string } | null',
			'worked_minutes: number',
			'reason: string',
			'surcharge_id: Uuid | null',
			'rate_bp: number',
			'status: DecisionStatus',
			'created_by_member_id: Uuid | null',
			'decided_by_member_id: Uuid | null',
			'decided_at: DateTime | null',
			'decision_comment: string | null',
			'created_at: DateTime',
			'updated_at: DateTime',
		]),
		uses: ['DecisionStatus', 'Uuid', 'IsoDate', 'DateTime'],
	},
	SupplierInvoiceLineAllocation: {
		source: block('interface', 'SupplierInvoiceLineAllocation', [
			'id: Uuid',
			'supplier_invoice_line_id: Uuid',
			'project_id: Uuid',
			'amount_cents: Cents',
		]),
		uses: ['Uuid', 'Cents'],
	},
}

function created(field: string, type: string): PayloadContract {
	return { fields: [`${field}: ${type}`], uses: [type] }
}

function updated(field: string, type: string): PayloadContract {
	return {
		fields: [
			`${field}: ${type}`,
			'changed_fields: string[]',
			`previous: Partial<${type}>`,
		],
		uses: [type],
	}
}

function transitioned(
	field: string,
	type: string,
	statusType: string,
	to: string,
): PayloadContract {
	return {
		fields: [`${field}: ${type}`, `from: ${statusType}`, `to: '${to}'`],
		uses: [type, statusType],
	}
}

function deleted(field: string): PayloadContract {
	return { fields: [`${field}: Uuid`], uses: ['Uuid'] }
}

const PAYLOADS: Record<string, PayloadContract> = {
	'quote.created': created('quote', 'Quote'),
	'quote.updated': updated('quote', 'Quote'),
	'quote.sent': transitioned('quote', 'Quote', 'QuoteStatus', 'SENT'),
	'quote.accepted': transitioned('quote', 'Quote', 'QuoteStatus', 'ACCEPTED'),
	'quote.declined': transitioned('quote', 'Quote', 'QuoteStatus', 'DECLINED'),
	'quote.cancelled': transitioned('quote', 'Quote', 'QuoteStatus', 'CANCELLED'),
	'quote.deleted': deleted('quote_id'),
	'invoice.created': created('invoice', 'Invoice'),
	'invoice.updated': updated('invoice', 'Invoice'),
	'invoice.issued': transitioned(
		'invoice',
		'Invoice',
		'InvoiceStatus',
		'ISSUED',
	),
	'invoice.partially_paid': transitioned(
		'invoice',
		'Invoice',
		'InvoiceStatus',
		'PARTIALLY_PAID',
	),
	'invoice.paid': transitioned('invoice', 'Invoice', 'InvoiceStatus', 'PAID'),
	'invoice.cancelled': transitioned(
		'invoice',
		'Invoice',
		'InvoiceStatus',
		'CANCELLED',
	),
	'invoice.deleted': deleted('invoice_id'),
	'invoice_payment.recorded': created('payment', 'InvoicePayment'),
	'invoice_payment.deleted': {
		fields: ['payment_id: Uuid', 'invoice_id: Uuid', 'deleted_by: Uuid'],
		uses: ['Uuid'],
	},
	'ticket.created': created('ticket', 'Ticket'),
	'ticket.transitioned': {
		fields: ['ticket: Ticket', 'from: TicketStatus', 'to: TicketStatus'],
		uses: ['Ticket', 'TicketStatus'],
	},
	'maintenance_intervention.proposed': created(
		'maintenance_intervention',
		'MaintenanceIntervention',
	),
	'maintenance_intervention.validated': created(
		'maintenance_intervention',
		'MaintenanceIntervention',
	),
	'maintenance_intervention.rejected': created(
		'maintenance_intervention',
		'MaintenanceIntervention',
	),
	'time_entry.started': created('time_entry', 'TimeEntry'),
	'time_entry.stopped': {
		fields: ['time_entry: TimeEntry', 'worked_minutes: number'],
		uses: ['TimeEntry'],
	},
	'time_entry.declared': {
		fields: ['time_entry: TimeEntry', 'worked_minutes: number'],
		uses: ['TimeEntry'],
	},
	'day.ended': {
		fields: ['day_log: DayLog', 'closed_entries: number'],
		uses: ['DayLog'],
	},
	'assignment_report.reported': created(
		'assignment_report',
		'AssignmentReport',
	),
	'assignment_report.applied': created('assignment_report', 'AssignmentReport'),
	'assignment_report.dismissed': created(
		'assignment_report',
		'AssignmentReport',
	),
	'supplier_invoice.received': created('supplier_invoice', 'SupplierInvoice'),
	'supplier_invoice.confirmed': transitioned(
		'supplier_invoice',
		'SupplierInvoice',
		'SupplierInvoiceStatus',
		'CONFIRMED',
	),
	'supplier_invoice.rejected': transitioned(
		'supplier_invoice',
		'SupplierInvoice',
		'SupplierInvoiceStatus',
		'REJECTED',
	),
	'supplier_invoice_line_allocation.recorded': created(
		'allocation',
		'SupplierInvoiceLineAllocation',
	),
	'member.invited': {
		fields: [
			'invitation_id: Uuid',
			'organization_id: Uuid',
			'member_id: Uuid',
			'expires_at: DateTime',
			'created_by_user_id: Uuid',
		],
		uses: ['Uuid', 'DateTime'],
	},
	'member.joined': {
		fields: [
			'member_id: Uuid',
			'organization_id: Uuid',
			'user_id: Uuid',
			'invitation_id: Uuid',
			'joined_at: DateTime',
		],
		uses: ['Uuid', 'DateTime'],
	},
	'customer.created': created('customer', 'Customer'),
	'deal.created': created('deal', 'Deal'),
	'deal.stage_changed': {
		fields: ['deal: Deal', 'from: DealStage', 'to: DealStage'],
		uses: ['Deal', 'DealStage'],
	},
	'absence.requested': created('absence', 'Absence'),
	'absence.approved': transitioned(
		'absence',
		'Absence',
		'DecisionStatus',
		'APPROVED',
	),
	'absence.rejected': transitioned(
		'absence',
		'Absence',
		'DecisionStatus',
		'REJECTED',
	),
	'absence.cancelled': transitioned(
		'absence',
		'Absence',
		'DecisionStatus',
		'CANCELLED',
	),
	'extra_period.requested': created('extra_period', 'ExtraPeriod'),
	'extra_period.approved': transitioned(
		'extra_period',
		'ExtraPeriod',
		'DecisionStatus',
		'APPROVED',
	),
	'extra_period.rejected': transitioned(
		'extra_period',
		'ExtraPeriod',
		'DecisionStatus',
		'REJECTED',
	),
	'extra_period.cancelled': transitioned(
		'extra_period',
		'ExtraPeriod',
		'DecisionStatus',
		'CANCELLED',
	),
	'member.removed': {
		fields: [
			'member_id: Uuid',
			'organization_id: Uuid',
			'user_id: Uuid',
			'display_name: string',
		],
		uses: ['Uuid'],
	},
}

export function payloadTypeName(eventName: string): string {
	const pascal = eventName
		.split(/[._]/)
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join('')
	return `${pascal}Payload`
}

function collectDeclarations(names: readonly string[], seen: Set<string>) {
	for (const name of names) {
		if (seen.has(name)) continue
		seen.add(name)
		collectDeclarations(DECLARATIONS[name]?.uses ?? [], seen)
	}
}

export function hasContract(eventName: string): boolean {
	return eventName in PAYLOADS
}

export function payloadContractFields(eventName: string): readonly string[] {
	return PAYLOADS[eventName]?.fields ?? []
}

export function eventContractSource(eventName: string): string {
	const payload = PAYLOADS[eventName]
	if (!payload) return ''
	const seen = new Set<string>()
	collectDeclarations(payload.uses, seen)
	const declarations = [...seen]
		.map((name) => DECLARATIONS[name]?.source)
		.filter((source): source is string => Boolean(source))
	return [
		block('export interface', payloadTypeName(eventName), payload.fields),
		...declarations,
	].join('\n\n')
}
