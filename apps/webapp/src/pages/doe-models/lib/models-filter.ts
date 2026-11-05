import type { DoeModel } from '#/hooks/use-doe-models'

export type ModelStatus = 'actifs' | 'archives'

export const DEFAULT_MODEL_STATUS: ModelStatus = 'actifs'

export function isModelStatus(value: unknown): value is ModelStatus {
	return value === 'actifs' || value === 'archives'
}

export type ModelsSearch = { statut?: 'archives' }

export function parseModelsSearch(raw: Record<string, unknown>): ModelsSearch {
	return raw.statut === 'archives' ? { statut: 'archives' } : {}
}

export function isArchived(model: Pick<DoeModel, 'archived_at'>): boolean {
	return Boolean(model.archived_at)
}

export function filterModels(
	models: readonly DoeModel[],
	status: ModelStatus,
): DoeModel[] {
	return models
		.filter((model) => isArchived(model) === (status === 'archives'))
		.sort((a, b) => a.name.localeCompare(b.name, 'fr'))
}

export function modelCounts(models: readonly DoeModel[]): {
	active: number
	archived: number
} {
	const archived = models.filter(isArchived).length
	return { active: models.length - archived, archived }
}

export interface ThemeCounts {
	total: number
	enabled: number
}

export function themeCounts(model: Pick<DoeModel, 'sections'>): ThemeCounts {
	return {
		total: model.sections.length,
		enabled: model.sections.filter((section) => section.enabled).length,
	}
}

export function themesLabel({ total, enabled }: ThemeCounts): string {
	if (total === 0) return 'Aucun thème'
	const themes = `${total} ${total > 1 ? 'thèmes' : 'thème'}`
	const active = `${enabled} ${enabled > 1 ? 'activés' : 'activé'}`
	return `${themes}, ${active}`
}
