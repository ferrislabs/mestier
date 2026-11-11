import { z } from 'zod'

export const tableSearchSchema = z.object({
	vue: z.uuid().optional().catch(undefined),
})

export type TableSearch = z.infer<typeof tableSearchSchema>
