import { describe, expect, it } from 'vitest'
import { buildSettingsNavGroups } from '#/pages/settings/nav'
import { SETTINGS_SECTIONS } from '#/pages/settings/registry'

describe('SETTINGS_SECTIONS', () => {
	it('chaque section apparaît exactement une fois dans les groupes de navigation', () => {
		const groups = buildSettingsNavGroups(SETTINGS_SECTIONS)

		const idsDuRegistre = SETTINGS_SECTIONS.map((section) => section.id).sort()
		const idsDesGroupes = groups
			.flatMap((group) => group.sections)
			.map((section) => section.id)
			.sort()

		expect(idsDesGroupes).toEqual(idsDuRegistre)
	})

	it('a un identifiant unique par section', () => {
		const ids = SETTINGS_SECTIONS.map((section) => section.id)

		expect(new Set(ids).size).toBe(ids.length)
	})

	it('inclut la section automatisation (#155), reachable par ancre', () => {
		expect(
			SETTINGS_SECTIONS.some((section) => section.id === 'automatisation'),
		).toBe(true)
	})

	it('groupe les modèles de projet, de DOE et la mise en page au même endroit', () => {
		expect(SETTINGS_SECTIONS.map((section) => section.id)).toContain('modeles')
		expect(SETTINGS_SECTIONS.map((section) => section.id)).not.toContain(
			'documents',
		)
	})
})
