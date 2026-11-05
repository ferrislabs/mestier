import { Boxes } from 'lucide-react'
import { describe, expect, it } from 'vitest'
import { buildSettingsNavGroups } from '#/pages/settings/nav'
import type { SettingsSection } from '#/pages/settings/types'

const section = (
	id: string,
	moduleId?: SettingsSection['moduleId'],
): SettingsSection => ({
	id,
	label: id,
	icon: Boxes,
	moduleId,
	Component: () => null,
})

describe('buildSettingsNavGroups', () => {
	it('place les sections sans module dans un groupe Général, en premier', () => {
		const groups = buildSettingsNavGroups([
			section('crm', 'crm'),
			section('organisation'),
		])

		expect(groups[0]?.label).toBe('Général')
		expect(groups[0]?.sections.map((s) => s.id)).toEqual(['organisation'])
	})

	it("conserve l'ordre de déclaration à l'intérieur d'un groupe", () => {
		const groups = buildSettingsNavGroups([
			section('organisation'),
			section('automatisation'),
		])

		expect(groups[0]?.sections.map((s) => s.id)).toEqual([
			'organisation',
			'automatisation',
		])
	})

	it('crée un groupe par module contributeur, libellé par le module', () => {
		const groups = buildSettingsNavGroups([
			section('organisation'),
			section('crm', 'crm'),
		])

		expect(groups.map((g) => g.label)).toEqual(['Général', 'CRM'])
	})

	it('ordonne les groupes de module selon le registre de modules', () => {
		const groups = buildSettingsNavGroups([
			section('rh-truc', 'hr'),
			section('crm-truc', 'crm'),
		])

		expect(groups.map((g) => g.label)).toEqual(['CRM', 'RH'])
	})

	it("n'émet aucun groupe pour un module sans section", () => {
		const groups = buildSettingsNavGroups([section('organisation')])

		expect(groups).toHaveLength(1)
	})

	it("n'émet pas de groupe Général si aucune section générale", () => {
		const groups = buildSettingsNavGroups([section('crm', 'crm')])

		expect(groups.map((g) => g.label)).toEqual(['CRM'])
	})
})
