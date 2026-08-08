import { describe, expect, it } from 'vitest'
import { MODULES } from '#/modules/registry'
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

	it('chaque section de configuration référencée par un module existe', () => {
		const ids = new Set(SETTINGS_SECTIONS.map((section) => section.id))

		const referencesOrphelines = MODULES.map(
			(module) => module.settingsSectionId,
		).filter((sectionId) => sectionId !== undefined && !ids.has(sectionId))

		expect(referencesOrphelines).toEqual([])
	})
})
