import { z } from 'zod'

export const projectTemplatesSearchSchema = z.object({
	archived: z.boolean().catch(false),
})

export type ProjectTemplatesSearch = z.infer<
	typeof projectTemplatesSearchSchema
>

export interface ProjectTemplateFormValues {
	name: string
	description: string
}

export const EMPTY_PROJECT_TEMPLATE_FORM: ProjectTemplateFormValues = {
	name: '',
	description: '',
}
