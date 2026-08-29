import { describe, expect, it } from 'vitest'
import type { PermissionName } from '#/hooks/use-permissions'
import {
	PERMISSION_CATALOG,
	permissionDescriptor,
	permissionsByArea,
} from '#/lib/permission-catalog'

const ALL_PERMISSION_NAMES: PermissionName[] = [
	'MANAGE_ORG',
	'MANAGE_MEMBERS',
	'MANAGE_ROLES',
	'MANAGE_CHANNELS',
	'MANAGE_WEBHOOKS',
	'VIEW_CHANNEL',
	'SEND_MESSAGES',
	'VIEW_PLANNING',
	'MANAGE_PLANNING',
	'VIEW_COST',
	'MANAGE_COST',
	'VIEW_REPORTS',
	'MANAGE_CUSTOMERS',
	'MANAGE_QUOTES',
	'MANAGE_REFERENCE',
]

describe('PERMISSION_CATALOG', () => {
	it('carries exactly one entry per bit the backend knows, no more no less', () => {
		const catalogNames = PERMISSION_CATALOG.map((p) => p.name).sort()
		const expected = [...ALL_PERMISSION_NAMES].sort()

		expect(catalogNames).toEqual(expected)
	})

	it('never repeats a bit name', () => {
		const names = PERMISSION_CATALOG.map((p) => p.name)
		expect(new Set(names).size).toBe(names.length)
	})

	it('gives every entry a non-empty label and description', () => {
		for (const entry of PERMISSION_CATALOG) {
			expect(entry.label.trim().length).toBeGreaterThan(0)
			expect(entry.description.trim().length).toBeGreaterThan(0)
		}
	})

	/** The binding requirement: granting the wrong cost bit is how payroll
	 * leaks, so the two must not read as interchangeable. */
	it('worries VIEW_COST and MANAGE_COST apart in wording', () => {
		const view = permissionDescriptor('VIEW_COST')
		const manage = permissionDescriptor('MANAGE_COST')

		expect(view?.label).not.toBe(manage?.label)
		expect(view?.description).not.toBe(manage?.description)
	})

	it('says on the spot what VIEW_REPORTS without VIEW_COST gives', () => {
		const reports = permissionDescriptor('VIEW_REPORTS')

		expect(reports?.description.toLowerCase()).toContain('heures')
	})
})

describe('permissionsByArea', () => {
	it('accounts for every permission across its areas exactly once', () => {
		const grouped = permissionsByArea()
		const total = grouped.reduce(
			(sum, group) => sum + group.permissions.length,
			0,
		)

		expect(total).toBe(PERMISSION_CATALOG.length)
	})

	it('lists the six areas the issue names', () => {
		const areas = permissionsByArea().map((group) => group.area)

		expect(areas).toEqual([
			'planning',
			'costs',
			'commercial',
			'reference',
			'chat',
			'administration',
		])
	})
})
