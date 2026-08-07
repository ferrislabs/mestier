import { describe, expect, it } from 'vitest'
import {
	absenceToDraft,
	draftToCreateAbsenceRequest,
	draftToUpdateAbsenceRequest,
	emptyAbsenceDraft,
	validateAbsenceDraft,
} from '#/pages/planning/lib/absences'

const TZ = 'Europe/Paris'

describe('emptyAbsenceDraft', () => {
	it('pré-remplit un congé journée entière sur un seul jour, pour un employé donné', () => {
		const draft = emptyAbsenceDraft('emp-1', '2026-08-10')
		expect(draft.employeeId).toBe('emp-1')
		expect(draft.kind).toBe('LEAVE')
		expect(draft.allDay).toBe(true)
		expect(draft.startDate).toBe('2026-08-10')
		expect(draft.endDate).toBe('2026-08-10')
	})
})

describe('validateAbsenceDraft', () => {
	it("signale l'employé manquant en création", () => {
		const draft = emptyAbsenceDraft('', '2026-08-10')
		const errors = validateAbsenceDraft(draft, { requireEmployee: true })
		expect(errors).toContain('Employé requis')
	})

	it("n'exige pas l'employé en édition", () => {
		const draft = emptyAbsenceDraft('emp-1', '2026-08-10')
		const errors = validateAbsenceDraft(draft, { requireEmployee: false })
		expect(errors).not.toContain('Employé requis')
	})

	it('accepte une absence journée entière sur un seul jour (fin == début)', () => {
		const draft = emptyAbsenceDraft('emp-1', '2026-08-10')
		expect(validateAbsenceDraft(draft)).toEqual([])
	})

	it('rejette une fin de journée entière avant le début', () => {
		const draft = {
			...emptyAbsenceDraft('emp-1', '2026-08-10'),
			endDate: '2026-08-09',
		}
		expect(validateAbsenceDraft(draft).length).toBeGreaterThan(0)
	})

	it("rejette un horaire invalide quand ce n'est pas journée entière", () => {
		const draft = {
			...emptyAbsenceDraft('emp-1', '2026-08-10'),
			allDay: false,
			startTime: 'nope',
			endTime: '18:00',
		}
		expect(validateAbsenceDraft(draft).length).toBeGreaterThan(0)
	})

	it('rejette une fin avant le début sur un créneau horaire', () => {
		const draft = {
			...emptyAbsenceDraft('emp-1', '2026-08-10'),
			allDay: false,
			startTime: '18:00',
			endTime: '08:00',
		}
		expect(validateAbsenceDraft(draft).length).toBeGreaterThan(0)
	})
})

describe('draftToCreateAbsenceRequest — journée entière', () => {
	it('construit starts_at/ends_at comme la fenêtre locale [00:00, 24:00) sur la plage de jours', () => {
		const draft = {
			...emptyAbsenceDraft('emp-1', '2026-08-10'),
			endDate: '2026-08-12',
			note: 'Congés été',
		}
		const request = draftToCreateAbsenceRequest(draft, TZ)

		expect(request).toEqual({
			employee_id: 'emp-1',
			kind: 'LEAVE',
			all_day: true,
			starts_at: new Date('2026-08-10T00:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-13T00:00:00+02:00').toISOString(),
			note: 'Congés été',
		})
	})

	it('renvoie null quand la validation échoue plutôt que de fabriquer un payload cassé', () => {
		const draft = { ...emptyAbsenceDraft('', '2026-08-10') }
		expect(draftToCreateAbsenceRequest(draft, TZ)).toBeNull()
	})
})

describe('draftToCreateAbsenceRequest — créneau horaire', () => {
	it('combine date et heure locales en instants ISO', () => {
		const draft = {
			...emptyAbsenceDraft('emp-1', '2026-08-10'),
			allDay: false,
			kind: 'SICK' as const,
			startTime: '09:00',
			endTime: '17:00',
			note: '',
		}
		const request = draftToCreateAbsenceRequest(draft, TZ)

		expect(request).toEqual({
			employee_id: 'emp-1',
			kind: 'SICK',
			all_day: false,
			starts_at: new Date('2026-08-10T09:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-10T17:00:00+02:00').toISOString(),
			note: null,
		})
	})
})

describe('draftToUpdateAbsenceRequest', () => {
	it("ne porte pas employee_id — l'API ne l'accepte pas en édition", () => {
		const draft = emptyAbsenceDraft('emp-1', '2026-08-10')
		const request = draftToUpdateAbsenceRequest(draft, TZ)
		expect(request).not.toHaveProperty('employee_id')
		expect(request).toMatchObject({ kind: 'LEAVE', all_day: true })
	})

	it('renvoie null quand la validation échoue', () => {
		const draft = {
			...emptyAbsenceDraft('emp-1', '2026-08-10'),
			endDate: '2026-08-01',
		}
		expect(draftToUpdateAbsenceRequest(draft, TZ)).toBeNull()
	})
})

describe('absenceToDraft — aller-retour', () => {
	it('reconstruit un draft journée entière cohérent avec le payload qui l’a produit', () => {
		const created = draftToCreateAbsenceRequest(
			{
				...emptyAbsenceDraft('emp-1', '2026-08-10'),
				endDate: '2026-08-12',
				kind: 'UNAVAILABLE',
				note: 'Formation',
			},
			TZ,
		)
		if (!created) throw new Error('expected a request')

		const draft = absenceToDraft(
			{
				employee_id: 'emp-1',
				absence_kind: created.kind,
				all_day: created.all_day ?? true,
				starts_at: created.starts_at,
				ends_at: created.ends_at,
				note: created.note ?? null,
			},
			TZ,
		)

		expect(draft).toMatchObject({
			employeeId: 'emp-1',
			kind: 'UNAVAILABLE',
			allDay: true,
			startDate: '2026-08-10',
			endDate: '2026-08-12',
			note: 'Formation',
		})
	})

	it('reconstruit un draft à créneau horaire cohérent avec le payload qui l’a produit', () => {
		const created = draftToCreateAbsenceRequest(
			{
				...emptyAbsenceDraft('emp-1', '2026-08-10'),
				allDay: false,
				startTime: '09:00',
				endTime: '17:00',
			},
			TZ,
		)
		if (!created) throw new Error('expected a request')

		const draft = absenceToDraft(
			{
				employee_id: 'emp-1',
				absence_kind: created.kind,
				all_day: created.all_day ?? false,
				starts_at: created.starts_at,
				ends_at: created.ends_at,
				note: created.note ?? null,
			},
			TZ,
		)

		expect(draft).toMatchObject({
			allDay: false,
			startDate: '2026-08-10',
			startTime: '09:00',
			endDate: '2026-08-10',
			endTime: '17:00',
		})
	})
})
