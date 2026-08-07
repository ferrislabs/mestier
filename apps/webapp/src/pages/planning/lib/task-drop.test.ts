import { describe, expect, it } from 'vitest'
import {
	assigneeRefFromResourceId,
	buildAssigneesForMove,
	buildAssigneesForRemoval,
	computeRemoveAssigneePatch,
	computeTaskDropPatch,
	shiftInstant,
} from '#/pages/planning/lib/task-drop'

const TZ = 'Europe/Paris'

describe('assigneeRefFromResourceId', () => {
	it('parse un resource_id employee', () => {
		expect(assigneeRefFromResourceId('employee:emp-1')).toEqual({
			kind: 'employee',
			employee_id: 'emp-1',
		})
	})

	it('parse un resource_id member', () => {
		expect(assigneeRefFromResourceId('member:user-9')).toEqual({
			kind: 'member',
			user_id: 'user-9',
		})
	})

	it('lève une erreur sur un kind inconnu — un bug amont, pas une donnée à avaler', () => {
		expect(() => assigneeRefFromResourceId('unknown:x')).toThrow()
	})
})

describe('shiftInstant', () => {
	it('renvoie la même valeur pour un décalage nul', () => {
		expect(shiftInstant('2026-08-10T08:00:00Z', 0, TZ)).toBe(
			'2026-08-10T08:00:00Z',
		)
	})

	it('décale de N jours en préservant l’heure locale', () => {
		const shifted = shiftInstant('2026-08-10T08:00:00+02:00', 2, TZ)
		expect(shifted).toBe(new Date('2026-08-12T08:00:00+02:00').toISOString())
	})

	it('décale correctement à travers un changement DST (fin octobre en Europe/Paris)', () => {
		// 2026-10-24 est en CEST (+02:00), 2026-10-27 est en CET (+01:00) —
		// franchir le changement d'heure doit conserver 08:00 heure locale.
		const shifted = shiftInstant('2026-10-24T08:00:00+02:00', 3, TZ)
		expect(shifted).toBe(new Date('2026-10-27T08:00:00+01:00').toISOString())
	})
})

describe('buildAssigneesForMove', () => {
	it('remplace la ressource source par la cible, en préservant les autres', () => {
		const result = buildAssigneesForMove(
			['emp-1', 'emp-2'],
			'employee:emp-1',
			'employee:emp-3',
		)
		expect(result).toEqual([
			{ kind: 'employee', employee_id: 'emp-3' },
			{ kind: 'employee', employee_id: 'emp-2' },
		])
	})

	it('convertit vers un member quand la cible est une ressource member-only', () => {
		const result = buildAssigneesForMove(
			['emp-1'],
			'employee:emp-1',
			'member:user-9',
		)
		expect(result).toEqual([{ kind: 'member', user_id: 'user-9' }])
	})

	it('ne duplique pas quand la cible est déjà assignée ailleurs sur le même chantier', () => {
		const result = buildAssigneesForMove(
			['emp-1', 'emp-2'],
			'employee:emp-1',
			'employee:emp-2',
		)
		expect(result).toEqual([{ kind: 'employee', employee_id: 'emp-2' }])
	})

	it('ne touche à rien quand source et cible sont la même ressource', () => {
		const result = buildAssigneesForMove(
			['emp-1', 'emp-2'],
			'employee:emp-1',
			'employee:emp-1',
		)
		expect(result).toEqual([
			{ kind: 'employee', employee_id: 'emp-1' },
			{ kind: 'employee', employee_id: 'emp-2' },
		])
	})
})

describe('buildAssigneesForRemoval', () => {
	it('retire la ressource visée et conserve les autres', () => {
		const result = buildAssigneesForRemoval(
			['emp-1', 'emp-2'],
			'employee:emp-1',
		)
		expect(result).toEqual([{ kind: 'employee', employee_id: 'emp-2' }])
	})

	it('ne change rien si la ressource ne figure pas dans la liste', () => {
		const result = buildAssigneesForRemoval(['emp-2'], 'employee:emp-1')
		expect(result).toEqual([{ kind: 'employee', employee_id: 'emp-2' }])
	})
})

describe('computeTaskDropPatch — jour seul', () => {
	it('ne porte que starts_at/ends_at décalés, avec la liste complète des assignés inchangée', () => {
		const result = computeTaskDropPatch({
			source: {
				entryId: 'wo-1',
				resourceId: 'employee:emp-1',
				date: '2026-08-10',
			},
			target: { resourceId: 'employee:emp-1', date: '2026-08-12' },
			entry: {
				startsAt: '2026-08-10T08:00:00+02:00',
				endsAt: '2026-08-10T10:00:00+02:00',
				employeeIds: ['emp-1'],
			},
			timeZone: TZ,
		})

		expect(result.changed).toBe(true)
		expect(result.body).toEqual({
			starts_at: new Date('2026-08-12T08:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-12T10:00:00+02:00').toISOString(),
			assignees: [{ kind: 'employee', employee_id: 'emp-1' }],
		})
	})
})

describe('computeTaskDropPatch — ligne seule', () => {
	it('ne porte que la liste des assignés, sans starts_at/ends_at', () => {
		const result = computeTaskDropPatch({
			source: {
				entryId: 'wo-1',
				resourceId: 'employee:emp-1',
				date: '2026-08-10',
			},
			target: { resourceId: 'employee:emp-2', date: '2026-08-10' },
			entry: {
				startsAt: '2026-08-10T08:00:00+02:00',
				endsAt: '2026-08-10T10:00:00+02:00',
				employeeIds: ['emp-1'],
			},
			timeZone: TZ,
		})

		expect(result.changed).toBe(true)
		expect(result.body).toEqual({
			assignees: [{ kind: 'employee', employee_id: 'emp-2' }],
		})
		expect(result.body.starts_at).toBeUndefined()
		expect(result.body.ends_at).toBeUndefined()
	})
})

describe('computeTaskDropPatch — jour et ligne à la fois', () => {
	it('porte starts_at/ends_at décalés et la nouvelle liste des assignés, en un seul objet', () => {
		const result = computeTaskDropPatch({
			source: {
				entryId: 'wo-1',
				resourceId: 'employee:emp-1',
				date: '2026-08-10',
			},
			target: { resourceId: 'employee:emp-2', date: '2026-08-11' },
			entry: {
				startsAt: '2026-08-10T08:00:00+02:00',
				endsAt: '2026-08-10T10:00:00+02:00',
				employeeIds: ['emp-1'],
			},
			timeZone: TZ,
		})

		expect(result.changed).toBe(true)
		expect(result.body).toEqual({
			starts_at: new Date('2026-08-11T08:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-11T10:00:00+02:00').toISOString(),
			assignees: [{ kind: 'employee', employee_id: 'emp-2' }],
		})
	})
})

describe('computeTaskDropPatch — drop sans effet', () => {
	it('ne change rien quand le drop atterrit sur la même ligne et la même date', () => {
		const result = computeTaskDropPatch({
			source: {
				entryId: 'wo-1',
				resourceId: 'employee:emp-1',
				date: '2026-08-10',
			},
			target: { resourceId: 'employee:emp-1', date: '2026-08-10' },
			entry: {
				startsAt: '2026-08-10T08:00:00+02:00',
				endsAt: '2026-08-10T10:00:00+02:00',
				employeeIds: ['emp-1'],
			},
			timeZone: TZ,
		})

		expect(result.changed).toBe(false)
		expect(result.body).toEqual({})
	})
})

describe('computeRemoveAssigneePatch', () => {
	it('produit la liste complète des assignés restants', () => {
		const result = computeRemoveAssigneePatch({
			employeeIds: ['emp-1', 'emp-2'],
			resourceId: 'employee:emp-1',
		})

		expect(result.changed).toBe(true)
		expect(result.body).toEqual({
			assignees: [{ kind: 'employee', employee_id: 'emp-2' }],
		})
	})

	it("ne change rien si la ressource visée n'est pas assignée", () => {
		const result = computeRemoveAssigneePatch({
			employeeIds: ['emp-2'],
			resourceId: 'employee:emp-1',
		})

		expect(result.changed).toBe(false)
		expect(result.body).toEqual({})
	})
})
