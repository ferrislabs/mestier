import type { BlockNoteEditor } from '@blocknote/core'
import {
	getDefaultSlashMenuItems,
	insertOrUpdateBlockForSlashMenu,
} from '@blocknote/core/extensions'
import type { DefaultReactSuggestionItem } from '@blocknote/react'
import {
	Code,
	Heading1,
	Heading2,
	Heading3,
	Image,
	Info,
	List,
	ListChecks,
	ListOrdered,
	Minus,
	Paperclip,
	Pilcrow,
	Quote,
	Table2,
} from 'lucide-react'
import type { ReactElement } from 'react'
import type { DocumentSchema } from '#/pages/knowledge-documents/feature/document-schema'
import {
	MENTION_KIND_LABELS,
	type MentionOption,
} from '#/pages/knowledge-documents/lib/mention-options'

type DocumentEditor = BlockNoteEditor<
	DocumentSchema['blockSchema'],
	DocumentSchema['inlineContentSchema'],
	DocumentSchema['styleSchema']
>

export const SLASH_KEYS = [
	'heading',
	'heading_2',
	'heading_3',
	'quote',
	'numbered_list',
	'bullet_list',
	'check_list',
	'paragraph',
	'code_block',
	'divider',
] as const

const ICON_SIZE = 18

const ICONS: Record<(typeof SLASH_KEYS)[number], ReactElement> = {
	heading: <Heading1 size={ICON_SIZE} />,
	heading_2: <Heading2 size={ICON_SIZE} />,
	heading_3: <Heading3 size={ICON_SIZE} />,
	quote: <Quote size={ICON_SIZE} />,
	numbered_list: <ListOrdered size={ICON_SIZE} />,
	bullet_list: <List size={ICON_SIZE} />,
	check_list: <ListChecks size={ICON_SIZE} />,
	paragraph: <Pilcrow size={ICON_SIZE} />,
	code_block: <Code size={ICON_SIZE} />,
	divider: <Minus size={ICON_SIZE} />,
}

const BLOCKS_GROUP = 'Blocs du document'

function isKnownKey(key: string): key is (typeof SLASH_KEYS)[number] {
	return (SLASH_KEYS as readonly string[]).includes(key)
}

export function documentSlashItems(
	editor: DocumentEditor,
): DefaultReactSuggestionItem[] {
	const standard = getDefaultSlashMenuItems(editor)
		.filter((item) => isKnownKey(item.key))
		.map((item): DefaultReactSuggestionItem => {
			const { key, ...rest } = item
			return { ...rest, icon: isKnownKey(key) ? ICONS[key] : undefined }
		})

	const insert = (type: 'callout' | 'image' | 'file' | 'tableView') => () => {
		insertOrUpdateBlockForSlashMenu(editor, { type })
	}

	return [
		...standard,
		{
			title: 'Encadré',
			subtext: 'Mettre un message en avant',
			aliases: ['callout', 'encadre', 'info', 'attention'],
			group: BLOCKS_GROUP,
			icon: <Info size={ICON_SIZE} />,
			onItemClick: insert('callout'),
		},
		{
			title: 'Image',
			subtext: 'Une image du Drive, de la photothèque ou nouvelle',
			aliases: ['image', 'photo'],
			group: BLOCKS_GROUP,
			icon: <Image size={ICON_SIZE} />,
			onItemClick: insert('image'),
		},
		{
			title: 'Fichier',
			subtext: 'Un fichier du Drive',
			aliases: ['fichier', 'document', 'pdf'],
			group: BLOCKS_GROUP,
			icon: <Paperclip size={ICON_SIZE} />,
			onItemClick: insert('file'),
		},
		{
			title: 'Vue de table',
			subtext: 'Les lignes d’une table, en lecture',
			aliases: ['table', 'tableau', 'vue'],
			group: BLOCKS_GROUP,
			icon: <Table2 size={ICON_SIZE} />,
			onItemClick: insert('tableView'),
		},
	]
}

export function mentionItems(
	editor: DocumentEditor,
	options: readonly MentionOption[],
): DefaultReactSuggestionItem[] {
	return options.map((option) => ({
		title: option.label,
		subtext: MENTION_KIND_LABELS[option.kind],
		onItemClick: () => {
			editor.insertInlineContent([
				{
					type: 'mention',
					props: {
						resourceType: option.kind,
						resourceId: option.id,
						label: option.label,
					},
				},
				' ',
			])
		},
	}))
}
