import { normalizeText } from '#/pages/automation/lib/text'

export const ALL_TAGS = 'all'

export type WorkflowSortKey = 'updated' | 'name-asc' | 'name-desc'

interface ListedWorkflow {
	name: string
	tags: readonly string[]
	updatedAt: string
}

export function collectTags(workflows: readonly ListedWorkflow[]): string[] {
	const tags = new Set(workflows.flatMap((workflow) => workflow.tags))
	return [...tags].sort((left, right) => left.localeCompare(right, 'fr'))
}

export function filterWorkflows<T extends ListedWorkflow>(
	workflows: readonly T[],
	query: string,
	tag: string,
): T[] {
	const needle = normalizeText(query)
	return workflows.filter((workflow) => {
		const matchesTag = tag === ALL_TAGS || workflow.tags.includes(tag)
		const matchesQuery =
			needle === '' || normalizeText(workflow.name).includes(needle)
		return matchesTag && matchesQuery
	})
}

export function sortWorkflows<T extends ListedWorkflow>(
	workflows: readonly T[],
	sort: WorkflowSortKey,
): T[] {
	const sorted = [...workflows]
	if (sort === 'updated')
		return sorted.sort(
			(left, right) =>
				new Date(right.updatedAt).getTime() -
				new Date(left.updatedAt).getTime(),
		)
	const direction = sort === 'name-asc' ? 1 : -1
	return sorted.sort(
		(left, right) => direction * left.name.localeCompare(right.name, 'fr'),
	)
}
