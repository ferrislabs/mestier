import { describe, expect, it } from 'vitest'
import {
	addAddress,
	EMPTY_HOST_COMPANY,
	emptyAddress,
	removeAddress,
	setMainAddress,
	toHostCompanyRequest,
	validateHostCompany,
} from '#/pages/cursus/host-companies/lib/host-companies'

const withLine = (line1: string, isMain = false) => ({
	...emptyAddress(),
	line1,
	isMain,
})

describe('address list helpers', () => {
	it('makes the first added address main', () => {
		const one = addAddress([])
		expect(one.map((a) => a.isMain)).toEqual([true])
		expect(addAddress(one).map((a) => a.isMain)).toEqual([true, false])
	})

	it('moves the main flag when another address is chosen', () => {
		const list = setMainAddress([withLine('a', true), withLine('b')], 1)
		expect(list.map((a) => a.isMain)).toEqual([false, true])
	})

	it('promotes the first remaining address when the main is removed', () => {
		const list = removeAddress(
			[withLine('a'), withLine('b', true), withLine('c')],
			1,
		)
		expect(list.map((a) => [a.line1, a.isMain])).toEqual([
			['a', true],
			['c', false],
		])
	})

	it('returns an empty list when the last address is removed', () => {
		expect(removeAddress([withLine('a', true)], 0)).toEqual([])
	})
})

describe('host company form', () => {
	it('requires a name and address lines', () => {
		expect(validateHostCompany(EMPTY_HOST_COMPANY)).toMatch(/obligatoire/)
		expect(
			validateHostCompany({
				...EMPTY_HOST_COMPANY,
				name: 'Roux',
				addresses: [withLine('  ', true)],
			}),
		).toMatch(/adresse/)
		expect(
			validateHostCompany({
				...EMPTY_HOST_COMPANY,
				name: 'Roux',
				groupEmployeeCount: 'x',
			}),
		).toMatch(/groupe/)
	})

	it('builds the payload with nulls and a single main address', () => {
		const body = toHostCompanyRequest({
			...EMPTY_HOST_COMPANY,
			name: ' Roux ',
			groupEmployeeCount: '40',
			missions: '**Menuiserie**',
			addresses: [
				{ ...withLine(' 1 rue A '), label: 'Siège', city: 'Lyon' },
				withLine('2 rue B'),
			],
		})
		expect(body.name).toBe('Roux')
		expect(body.group_employee_count).toBe(40)
		expect(body.employee_count).toBeNull()
		expect(body.missions).toBe('**Menuiserie**')
		expect(body.addresses).toEqual([
			{
				label: 'Siège',
				line1: '1 rue A',
				line2: null,
				postal_code: null,
				city: 'Lyon',
				is_main: true,
			},
			{
				label: null,
				line1: '2 rue B',
				line2: null,
				postal_code: null,
				city: null,
				is_main: false,
			},
		])
	})
})
