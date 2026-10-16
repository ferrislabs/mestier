import type { Schemas } from '#/api/api.client'
import {
	type CadenceDraft,
	cadenceError,
	cadenceToDraft,
	emptyCadenceDraft,
	toCadenceRequest,
} from '#/pages/maintenance/lib/cadence-form'
import { isoWeeksInYear } from '#/pages/maintenance/lib/weeks'
import {
	type ContractStatus,
	DEFAULT_FIRST_WEEK,
	DEFAULT_LAST_WEEK,
	type MaintenanceContract,
} from '#/pages/maintenance/types'
import { centsToEuros, eurosToCents } from '#/pages/quotes/types'

export interface LineDraft {
	/** Stable across edits, for React; never sent. */
	key: string
	/** The API id of an existing line. An id keeps the line and its visits. */
	id: string | null
	label: string
	/** Minutes per visit, as typed. */
	minutes: string
	cadence: CadenceDraft
}

export interface ContractDraft {
	/** `null` while creating. */
	id: string | null
	customerId: string
	customerContextId: string
	reference: string
	year: string
	firstWeek: string
	lastWeek: string
	managerMemberId: string
	/** Euros, as typed. */
	amount: string
	notes: string
	/** Not on the form, but a `PUT` replaces everything: sent back as it was. */
	quoteId: string | null
	status: ContractStatus
	lines: LineDraft[]
}

let lineCounter = 0

export function emptyLineDraft(): LineDraft {
	lineCounter += 1

	return {
		key: `line-${lineCounter}`,
		id: null,
		label: '',
		minutes: '',
		cadence: emptyCadenceDraft(),
	}
}

export function emptyContractDraft(year: number): ContractDraft {
	return {
		id: null,
		customerId: '',
		customerContextId: '',
		reference: '',
		year: String(year),
		firstWeek: String(DEFAULT_FIRST_WEEK),
		lastWeek: String(DEFAULT_LAST_WEEK),
		managerMemberId: '',
		amount: '',
		notes: '',
		quoteId: null,
		status: 'ACTIVE',
		lines: [emptyLineDraft()],
	}
}

export function contractToDraft(contract: MaintenanceContract): ContractDraft {
	return {
		id: contract.id,
		customerId: contract.customer_id,
		customerContextId: contract.customer_context_id,
		reference: contract.reference ?? '',
		year: String(contract.season.year),
		firstWeek: String(contract.season.first_week),
		lastWeek: String(contract.season.last_week),
		managerMemberId: contract.manager_member_id ?? '',
		amount:
			contract.amount_cents == null ? '' : centsToEuros(contract.amount_cents),
		notes: contract.notes ?? '',
		quoteId: contract.quote_id ?? null,
		status: contract.status,
		lines: contract.lines.map((line) => {
			const draft = emptyLineDraft()

			return {
				...draft,
				id: line.id,
				label: line.label,
				minutes:
					line.minutes_per_intervention == null
						? ''
						: String(line.minutes_per_intervention),
				cadence: cadenceToDraft(line.cadence),
			}
		}),
	}
}

export interface ContractIssues {
	customer?: string
	site?: string
	season?: string
	amount?: string
	/** By line `key`. */
	lines: Record<string, string>
}

const MONEY = /^\d+([.,]\d{1,2})?$/

function wholeNumber(value: string): number | null {
	return /^\d+$/.test(value.trim()) ? Number(value.trim()) : null
}

/** The season the form describes, or why it does not describe one. */
export function readSeason(
	draft: Pick<ContractDraft, 'year' | 'firstWeek' | 'lastWeek'>,
): { year: number; firstWeek: number; lastWeek: number } | string {
	const year = wholeNumber(draft.year)
	const firstWeek = wholeNumber(draft.firstWeek)
	const lastWeek = wholeNumber(draft.lastWeek)
	if (year === null || year < 2000 || year > 2100) {
		return "L'année doit être comprise entre 2000 et 2100."
	}
	if (firstWeek === null || lastWeek === null) {
		return 'Indiquez la première et la dernière semaine.'
	}
	if (firstWeek < 1 || lastWeek < 1 || firstWeek > 53 || lastWeek > 53) {
		return 'Les semaines vont de 1 à 53.'
	}
	if (lastWeek < firstWeek) {
		return 'La dernière semaine ne peut pas précéder la première.'
	}
	if (lastWeek > isoWeeksInYear(year)) {
		return `${year} n'a que ${isoWeeksInYear(year)} semaines.`
	}

	return { year, firstWeek, lastWeek }
}

export function contractIssues(draft: ContractDraft): ContractIssues {
	const issues: ContractIssues = { lines: {} }

	// The customer and the site are fixed once the contract exists (the API
	// answers 409 to a change), so they are only checked on creation.
	if (draft.id === null) {
		if (!draft.customerId) issues.customer = 'Choisissez un client.'
		else if (!draft.customerContextId) issues.site = 'Choisissez un site.'
	}

	const season = readSeason(draft)
	if (typeof season === 'string') issues.season = season

	const amount = draft.amount.trim().replace(/\s/g, '')
	if (amount && !MONEY.test(amount)) {
		issues.amount = 'Un montant en euros, par exemple 1 250,00.'
	}

	for (const line of draft.lines) {
		if (!line.label.trim()) {
			issues.lines[line.key] = 'Donnez un nom à la prestation.'
			continue
		}
		const minutes = line.minutes.trim()
		if (minutes && (wholeNumber(minutes) === null || Number(minutes) < 1)) {
			issues.lines[line.key] = 'La durée est un nombre de minutes.'
			continue
		}
		// Windows are checked against the season being typed. While the
		// season itself is invalid, that error is the one to fix first.
		const weeks =
			typeof season === 'string'
				? { firstWeek: 1, lastWeek: 53 }
				: { firstWeek: season.firstWeek, lastWeek: season.lastWeek }
		const cadence = cadenceError(line.cadence, weeks)
		if (cadence) issues.lines[line.key] = cadence
	}

	return issues
}

export function hasIssues(issues: ContractIssues): boolean {
	return (
		Boolean(issues.customer || issues.site || issues.season || issues.amount) ||
		Object.keys(issues.lines).length > 0
	)
}

/** Only call once {@link contractIssues} has nothing to say. */
export function toContractBody(draft: ContractDraft): Schemas.ContractBody {
	const season = readSeason(draft)
	if (typeof season === 'string') throw new Error(season)
	const amount = draft.amount.trim().replace(/\s/g, '')

	return {
		reference: draft.reference.trim() || null,
		season: {
			year: season.year,
			first_week: season.firstWeek,
			last_week: season.lastWeek,
		},
		manager_member_id: draft.managerMemberId || null,
		amount_cents: amount ? eurosToCents(amount) : null,
		quote_id: draft.quoteId,
		notes: draft.notes.trim() || null,
		status: draft.status,
		lines: draft.lines.map((line) => ({
			id: line.id,
			label: line.label.trim(),
			cadence: toCadenceRequest(line.cadence),
			minutes_per_intervention: line.minutes.trim()
				? Number(line.minutes)
				: null,
		})),
	}
}

/**
 * The `PUT` that activates a contract. It replaces everything, so every field
 * goes back as the API gave it (not through the form, which trims and
 * reformats), with `ACTIVE` as the only change.
 */
export function activationBody(
	contract: MaintenanceContract,
): Schemas.ContractBody {
	return {
		reference: contract.reference ?? null,
		season: {
			year: contract.season.year,
			first_week: contract.season.first_week,
			last_week: contract.season.last_week,
		},
		manager_member_id: contract.manager_member_id ?? null,
		amount_cents: contract.amount_cents ?? null,
		quote_id: contract.quote_id ?? null,
		notes: contract.notes ?? null,
		status: 'ACTIVE',
		lines: contract.lines.map((line) => ({
			id: line.id,
			label: line.label,
			cadence: line.cadence,
			minutes_per_intervention: line.minutes_per_intervention ?? null,
		})),
	}
}

export function toCreateRequest(
	draft: ContractDraft,
): Schemas.CreateMaintenanceContractRequest {
	return {
		...toContractBody(draft),
		customer_id: draft.customerId,
		customer_context_id: draft.customerContextId,
	}
}

/** "Dupliquer pour 2027". */
export function duplicateLabel(contract: Pick<MaintenanceContract, 'season'>) {
	return `Dupliquer pour ${contract.season.year + 1}`
}
