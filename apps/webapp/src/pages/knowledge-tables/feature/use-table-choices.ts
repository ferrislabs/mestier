import { useCallback, useMemo } from 'react'
import { useChatDirectory } from '#/hooks/use-chat'
import { useDriveFileChoices } from '#/hooks/use-knowledge-tables'
import { useResourceOptions } from '#/pages/knowledge/feature/use-resource-options'
import type {
	Choice,
	LinkKind,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'

const NO_CHOICES: Choice[] = []

export function useTableChoices(
	organizationId: string,
	columns: readonly TableColumn[],
): (column: TableColumn) => Choice[] {
	const hasFiles = columns.some((column) => column.kind === 'file')
	const resources = useResourceOptions(organizationId)
	const files = useDriveFileChoices(organizationId, hasFiles)
	const directory = useChatDirectory(organizationId)

	const links = useMemo(() => {
		const byKind = new Map<LinkKind, Choice[]>()
		for (const option of resources) {
			const list = byKind.get(option.type) ?? []
			list.push({ id: option.id, label: option.label })
			byKind.set(option.type, list)
		}
		return byKind
	}, [resources])
	const fileChoices = useMemo(
		() =>
			(files.data ?? []).map((file) => ({ id: file.id, label: file.title })),
		[files.data],
	)
	const memberChoices = useMemo(
		() =>
			(directory.data ?? []).map((entry) => ({
				id: entry.user_id,
				label: entry.display_name,
			})),
		[directory.data],
	)

	return useCallback(
		(column: TableColumn) => {
			switch (column.kind) {
				case 'link':
					return links.get(column.config.link_kind ?? 'customer') ?? NO_CHOICES
				case 'file':
					return fileChoices
				case 'member':
					return memberChoices
				default:
					return NO_CHOICES
			}
		},
		[links, fileChoices, memberChoices],
	)
}
