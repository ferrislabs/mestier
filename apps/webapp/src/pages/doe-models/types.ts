import { z } from 'zod'
import type { MakerLabel, PhotosPerPage, SectionKind } from '#/pages/doe/types'

export const doeModelsSearchSchema = z.object({
	archived: z.boolean().catch(false),
})

export type DoeModelsSearch = z.infer<typeof doeModelsSearchSchema>

export interface ModelSectionDraft {
	key: string
	title: string
	enabled: boolean
	kind: SectionKind
	makerLabel: MakerLabel
	perPage: PhotosPerPage
	count: number
}

export interface ModelDraft {
	id: string | null
	name: string
	description: string
	sections: ModelSectionDraft[]
}
