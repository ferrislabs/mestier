import { EditorContent, useEditor } from '@tiptap/react'
import { useEffect, useMemo } from 'react'
import { cn } from '#/lib/utils'
import { createExtensions, proseClassName } from './extensions'

type RichTextViewProps = {
	markdown: string
}

export function RichTextView({ markdown }: RichTextViewProps) {
	const extensions = useMemo(() => createExtensions(), [])
	const editor = useEditor({
		extensions,
		content: markdown,
		contentType: 'markdown',
		editable: false,
		immediatelyRender: true,
	})

	useEffect(() => {
		if (editor && editor.getMarkdown() !== markdown) {
			editor.commands.setContent(markdown, { contentType: 'markdown' })
		}
	}, [editor, markdown])

	if (markdown.trim() === '') {
		return null
	}

	return (
		<EditorContent
			editor={editor}
			className={cn('text-sm', proseClassName)}
			data-slot="rich-text-view"
		/>
	)
}
