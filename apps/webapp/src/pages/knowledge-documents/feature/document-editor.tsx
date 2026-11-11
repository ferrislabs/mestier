import '@blocknote/shadcn/style.css'
import type { PartialBlock } from '@blocknote/core'
import { filterSuggestionItems } from '@blocknote/core/extensions'
import { fr } from '@blocknote/core/locales'
import {
	AddBlockButton,
	BasicTextStyleButton,
	BlockTypeSelect,
	blockTypeSelectItems,
	CreateLinkButton,
	DragHandleButton,
	DragHandleMenu,
	FormattingToolbar,
	FormattingToolbarController,
	NestBlockButton,
	RemoveBlockItem,
	SideMenu,
	SideMenuController,
	SuggestionMenuController,
	UnnestBlockButton,
	useBlockNoteEditor,
	useCreateBlockNote,
} from '@blocknote/react'
import { BlockNoteView } from '@blocknote/shadcn'
import { useMemo } from 'react'
import { useResourceOptions } from '#/pages/knowledge/feature/use-resource-options'
import {
	type DocumentEnvironment,
	DocumentEnvironmentProvider,
} from '#/pages/knowledge-documents/feature/document-environment'
import {
	documentSlashItems,
	mentionItems,
} from '#/pages/knowledge-documents/feature/document-menus'
import {
	type DocumentSchema,
	documentSchema,
} from '#/pages/knowledge-documents/feature/document-schema'
import {
	type EditorBlock,
	toDocumentContent,
} from '#/pages/knowledge-documents/lib/blocks'
import type { DocumentContent } from '#/pages/knowledge-documents/lib/document-api-types'
import {
	filterMentionOptions,
	type MentionOption,
} from '#/pages/knowledge-documents/lib/mention-options'

const MAX_HEADING_ITEM_LEVEL = 3

type InitialBlocks = PartialBlock<
	DocumentSchema['blockSchema'],
	DocumentSchema['inlineContentSchema'],
	DocumentSchema['styleSchema']
>[]

function DocumentToolbar() {
	const editor = useBlockNoteEditor()
	const items = blockTypeSelectItems(editor).filter(
		(item) =>
			item.type !== 'toggleListItem' &&
			(item.type !== 'heading' ||
				Number(item.props?.level ?? 1) <= MAX_HEADING_ITEM_LEVEL),
	)
	return (
		<FormattingToolbar>
			<BlockTypeSelect key="block-type" items={items} />
			<BasicTextStyleButton key="bold" basicTextStyle="bold" />
			<BasicTextStyleButton key="italic" basicTextStyle="italic" />
			<BasicTextStyleButton key="underline" basicTextStyle="underline" />
			<BasicTextStyleButton key="strike" basicTextStyle="strike" />
			<BasicTextStyleButton key="code" basicTextStyle="code" />
			<NestBlockButton key="nest" />
			<UnnestBlockButton key="unnest" />
			<CreateLinkButton key="link" />
		</FormattingToolbar>
	)
}

function DocumentDragHandleMenu() {
	return (
		<DragHandleMenu>
			<RemoveBlockItem>Supprimer</RemoveBlockItem>
		</DragHandleMenu>
	)
}

function useMentionOptions(organizationId: string): MentionOption[] {
	const options = useResourceOptions(organizationId)
	return useMemo(
		() =>
			options.map((option) => ({
				kind: option.type,
				id: option.id,
				label: option.label,
			})),
		[options],
	)
}

interface DocumentEditorProps {
	initialBlocks: EditorBlock[]
	editable: boolean
	environment: DocumentEnvironment
	onChange: (read: () => DocumentContent) => void
}

export function DocumentEditor({
	initialBlocks,
	editable,
	environment,
	onChange,
}: DocumentEditorProps) {
	const mentionOptions = useMentionOptions(environment.organizationId)
	const editor = useCreateBlockNote({
		schema: documentSchema,
		dictionary: fr,
		initialContent:
			initialBlocks.length > 0
				? (initialBlocks as unknown as InitialBlocks)
				: undefined,
	})

	return (
		<DocumentEnvironmentProvider value={environment}>
			<BlockNoteView
				editor={editor}
				editable={editable}
				formattingToolbar={false}
				slashMenu={false}
				sideMenu={false}
				emojiPicker={false}
				filePanel={false}
				tableHandles={false}
				onChange={() =>
					onChange(() =>
						toDocumentContent(editor.document as unknown as EditorBlock[]),
					)
				}
			>
				<FormattingToolbarController formattingToolbar={DocumentToolbar} />
				<SideMenuController
					sideMenu={() => (
						<SideMenu>
							<AddBlockButton />
							<DragHandleButton dragHandleMenu={DocumentDragHandleMenu} />
						</SideMenu>
					)}
				/>
				<SuggestionMenuController
					triggerCharacter="/"
					getItems={async (query) =>
						filterSuggestionItems(documentSlashItems(editor), query)
					}
				/>
				<SuggestionMenuController
					triggerCharacter="@"
					getItems={async (query) =>
						mentionItems(editor, filterMentionOptions(mentionOptions, query))
					}
				/>
			</BlockNoteView>
		</DocumentEnvironmentProvider>
	)
}
