import { describe, expect, it } from 'vitest'
import {
	ADMIN_SECTIONS,
	adminGuard,
} from '#/pages/admin-layout/lib/admin-sections'

describe('ADMIN_SECTIONS', () => {
	it('lists the four sections, news keeping its address', () => {
		expect(
			ADMIN_SECTIONS.map((section) => [section.label, section.to]),
		).toEqual([
			['Vue d’ensemble', '/admin'],
			['Organisations', '/admin/organisations'],
			['Utilisateurs', '/admin/utilisateurs'],
			['News', '/admin/news'],
		])
	})
})

describe('adminGuard', () => {
	it('waits while the access is loading', () => {
		expect(adminGuard(true, undefined)).toBe('loading')
	})

	it('allows a caller with access', () => {
		expect(adminGuard(false, true)).toBe('allowed')
	})

	it('denies a caller without access or when the read failed', () => {
		expect(adminGuard(false, false)).toBe('denied')
		expect(adminGuard(false, undefined)).toBe('denied')
	})
})
