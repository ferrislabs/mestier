import { photoSummary } from '#/pages/doe/lib/photos'
import { enabledNumberedSections, tabLabel } from '#/pages/doe/lib/sections'
import type { DoeForm, DoeSection, SectionKind } from '#/pages/doe/types'

export const INFO_TAB = 'informations'
export const PLAN_TAB = 'plan'

export type RubriqueKind = 'informations' | 'plan' | SectionKind

export interface Rubrique {
	id: string
	kind: RubriqueKind
	label: string
	title: string
	description: string
	summary: string
	section: DoeSection | null
	number: number | null
}

export const SECTION_DESCRIPTIONS: Record<SectionKind, string> = {
	supplies:
		'Fournitures du thème, avec leur fabricant, leurs photos et leurs fiches techniques.',
	photos: 'Photos du thème, mises en page selon le nombre par page.',
	plans: 'Plans du thème, un fichier par plan.',
	blank_pages: 'Pages laissées vides pour les notes du client.',
}

function counted(count: number, one: string, many: string): string {
	return `${count} ${count > 1 ? many : one}`
}

export function sectionSummary(section: DoeSection): string {
	const content = section.content
	switch (content.kind) {
		case 'supplies':
			return content.supplies.length === 0
				? 'Aucune fourniture'
				: counted(content.supplies.length, 'fourniture', 'fournitures')
		case 'photos':
			return content.photos.length === 0
				? 'Aucune photo'
				: photoSummary(content.photos.length, content.per_page)
		case 'plans':
			return content.plans.length === 0
				? 'Aucun plan'
				: counted(content.plans.length, 'plan', 'plans')
		case 'blank_pages':
			return counted(content.count, 'page libre', 'pages libres')
	}
}

export function infoSummary(form: Pick<DoeForm, 'parties'>): string {
	const named = form.parties.filter((party) => party.name.trim() !== '').length
	if (named === 0) return 'Aucun intervenant renseigné'
	return named > 1
		? `${named} intervenants renseignés`
		: '1 intervenant renseigné'
}

export function planSummary(sections: readonly DoeSection[]): string {
	if (sections.length === 0) return 'Aucun thème'
	const enabled = sections.filter((section) => section.enabled).length
	return `${enabled} sur ${sections.length} ${sections.length > 1 ? 'thèmes' : 'thème'} inclus`
}

export function rubriques(form: DoeForm): Rubrique[] {
	const fixed: Rubrique[] = [
		{
			id: INFO_TAB,
			kind: 'informations',
			label: 'Informations',
			title: 'Informations',
			description:
				'Chantier, photo de couverture et intervenants imprimés dans le DOE.',
			summary: infoSummary(form),
			section: null,
			number: null,
		},
		{
			id: PLAN_TAB,
			kind: 'plan',
			label: 'Plan du DOE',
			title: 'Plan du DOE',
			description:
				'Thèmes inclus dans le DOE, leur ordre et leur numérotation.',
			summary: planSummary(form.sections),
			section: null,
			number: null,
		},
	]
	const themes = enabledNumberedSections(form.sections).map(
		({ section, number }): Rubrique => ({
			id: section.id,
			kind: section.content.kind,
			label: `${number}. ${tabLabel(section.title)}`,
			title: `${number}. ${section.title}`,
			description: SECTION_DESCRIPTIONS[section.content.kind],
			summary: sectionSummary(section),
			section,
			number,
		}),
	)
	return [...fixed, ...themes]
}

export function resolveTab(
	raw: string | undefined,
	sections: readonly DoeSection[],
): string {
	if (raw === undefined || raw === INFO_TAB) return INFO_TAB
	if (raw === PLAN_TAB) return PLAN_TAB
	const found = sections.find((section) => section.id === raw)
	return found?.enabled ? raw : INFO_TAB
}

export type DoeSearch = { onglet?: string }

export function validateDoeSearch(raw: Record<string, unknown>): DoeSearch {
	const value = typeof raw.onglet === 'number' ? String(raw.onglet) : raw.onglet
	return typeof value === 'string' && value.trim() !== ''
		? { onglet: value }
		: {}
}

export function tabSearch(tab: string): DoeSearch {
	return tab === INFO_TAB ? {} : { onglet: tab }
}
