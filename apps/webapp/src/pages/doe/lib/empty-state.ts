export type CreationMode = 'pick-model' | 'from-starter'

export interface ModelOption {
	id: string
	archived_at?: string | null
}

export function activeModels<T extends ModelOption>(models: T[]): T[] {
	return models.filter((model) => !model.archived_at)
}

export function chooseCreationMode(models: ModelOption[]): CreationMode {
	return activeModels(models).length > 0 ? 'pick-model' : 'from-starter'
}
