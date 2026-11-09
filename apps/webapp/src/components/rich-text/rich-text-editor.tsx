import type { Editor } from '@tiptap/react'
import { EditorContent, useEditor, useEditorState } from '@tiptap/react'
import {
	Bold,
	Heading3,
	Italic,
	Link as LinkIcon,
	List,
	ListOrdered,
	Redo2,
	Undo2,
} from 'lucide-react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { Toggle } from '#/components/ui/toggle'
import { cn } from '#/lib/utils'
import { createExtensions, proseClassName } from './extensions'

type RichTextEditorProps = {
	value: string
	onChange: (markdown: string) => void
	placeholder?: string
	disabled?: boolean
	id?: string
	'aria-label'?: string
}

type ToolbarProps = {
	editor: Editor
	disabled: boolean
}

function LinkControl({ editor, disabled }: ToolbarProps) {
	const [open, setOpen] = useState(false)
	const [url, setUrl] = useState('')
	const active = useEditorState({
		editor,
		selector: (state) => state.editor.isActive('link'),
	})

	function handleOpenChange(next: boolean) {
		if (next) {
			setUrl((editor.getAttributes('link').href as string | undefined) ?? '')
		}
		setOpen(next)
	}

	function apply() {
		const href = url.trim()
		const chain = editor.chain().focus().extendMarkRange('link')
		if (href === '') {
			chain.unsetLink().run()
		} else {
			chain.setLink({ href }).run()
		}
		setOpen(false)
	}

	function remove() {
		editor.chain().focus().extendMarkRange('link').unsetLink().run()
		setOpen(false)
	}

	return (
		<Popover open={open} onOpenChange={handleOpenChange}>
			<PopoverTrigger asChild>
				<Toggle
					size="sm"
					pressed={active}
					disabled={disabled}
					aria-label="Lien"
					type="button"
				>
					<LinkIcon />
				</Toggle>
			</PopoverTrigger>
			<PopoverContent className="flex w-80 flex-col gap-2 p-3" align="start">
				<Input
					type="url"
					value={url}
					placeholder="https://"
					aria-label="Adresse du lien"
					onChange={(event) => setUrl(event.target.value)}
					onKeyDown={(event) => {
						if (event.key === 'Enter') {
							event.preventDefault()
							apply()
						}
					}}
				/>
				<div className="flex justify-end gap-2">
					{active && (
						<Button type="button" variant="ghost" size="sm" onClick={remove}>
							Retirer
						</Button>
					)}
					<Button type="button" size="sm" onClick={apply}>
						Appliquer
					</Button>
				</div>
			</PopoverContent>
		</Popover>
	)
}

function Toolbar({ editor, disabled }: ToolbarProps) {
	const state = useEditorState({
		editor,
		selector: (snapshot) => ({
			bold: snapshot.editor.isActive('bold'),
			italic: snapshot.editor.isActive('italic'),
			heading: snapshot.editor.isActive('heading', { level: 3 }),
			bulletList: snapshot.editor.isActive('bulletList'),
			orderedList: snapshot.editor.isActive('orderedList'),
			canUndo: snapshot.editor.can().undo(),
			canRedo: snapshot.editor.can().redo(),
		}),
	})

	return (
		<div
			role="toolbar"
			aria-label="Mise en forme"
			className="flex flex-wrap items-center gap-0.5 border-b border-input p-1"
		>
			<Toggle
				size="sm"
				type="button"
				aria-label="Gras"
				pressed={state.bold}
				disabled={disabled}
				onPressedChange={() => editor.chain().focus().toggleBold().run()}
			>
				<Bold />
			</Toggle>
			<Toggle
				size="sm"
				type="button"
				aria-label="Italique"
				pressed={state.italic}
				disabled={disabled}
				onPressedChange={() => editor.chain().focus().toggleItalic().run()}
			>
				<Italic />
			</Toggle>
			<Toggle
				size="sm"
				type="button"
				aria-label="Titre"
				pressed={state.heading}
				disabled={disabled}
				onPressedChange={() =>
					editor.chain().focus().toggleHeading({ level: 3 }).run()
				}
			>
				<Heading3 />
			</Toggle>
			<Toggle
				size="sm"
				type="button"
				aria-label="Liste à puces"
				pressed={state.bulletList}
				disabled={disabled}
				onPressedChange={() => editor.chain().focus().toggleBulletList().run()}
			>
				<List />
			</Toggle>
			<Toggle
				size="sm"
				type="button"
				aria-label="Liste numérotée"
				pressed={state.orderedList}
				disabled={disabled}
				onPressedChange={() => editor.chain().focus().toggleOrderedList().run()}
			>
				<ListOrdered />
			</Toggle>
			<LinkControl editor={editor} disabled={disabled} />
			<span className="mx-1 h-5 w-px bg-border" aria-hidden="true" />
			<Button
				type="button"
				variant="ghost"
				size="icon-sm"
				aria-label="Annuler"
				disabled={disabled || !state.canUndo}
				onClick={() => editor.chain().focus().undo().run()}
			>
				<Undo2 />
			</Button>
			<Button
				type="button"
				variant="ghost"
				size="icon-sm"
				aria-label="Rétablir"
				disabled={disabled || !state.canRedo}
				onClick={() => editor.chain().focus().redo().run()}
			>
				<Redo2 />
			</Button>
		</div>
	)
}

export function RichTextEditor({
	value,
	onChange,
	placeholder,
	disabled = false,
	id,
	'aria-label': ariaLabel,
}: RichTextEditorProps) {
	const extensions = useMemo(() => createExtensions(placeholder), [placeholder])
	const onChangeRef = useRef(onChange)
	onChangeRef.current = onChange
	const lastEmitted = useRef(value)

	const editor = useEditor({
		extensions,
		content: value,
		contentType: 'markdown',
		editable: !disabled,
		immediatelyRender: true,
		editorProps: {
			attributes: {
				...(id ? { id } : {}),
				...(ariaLabel ? { 'aria-label': ariaLabel } : {}),
				role: 'textbox',
				'aria-multiline': 'true',
				class: 'min-h-24 px-3 py-2 text-sm outline-none',
			},
		},
		onUpdate: ({ editor: current }) => {
			const markdown = current.getMarkdown()
			if (markdown === lastEmitted.current) {
				return
			}
			lastEmitted.current = markdown
			onChangeRef.current(markdown)
		},
	})

	useEffect(() => {
		if (!editor || value === lastEmitted.current) {
			return
		}
		lastEmitted.current = value
		editor.commands.setContent(value, {
			contentType: 'markdown',
			emitUpdate: false,
		})
	}, [editor, value])

	useEffect(() => {
		editor?.setEditable(!disabled)
	}, [editor, disabled])

	if (!editor) {
		return null
	}

	return (
		<div
			data-slot="rich-text-editor"
			aria-disabled={disabled || undefined}
			className={cn(
				'w-full rounded-md border border-input bg-transparent shadow-xs transition-[color,box-shadow] dark:bg-input/30',
				'focus-within:border-ring focus-within:ring-[3px] focus-within:ring-ring/50',
				disabled && 'cursor-not-allowed opacity-50',
			)}
		>
			<Toolbar editor={editor} disabled={disabled} />
			<EditorContent
				editor={editor}
				className={cn(
					proseClassName,
					'[&_.is-editor-empty:first-child::before]:pointer-events-none [&_.is-editor-empty:first-child::before]:float-left [&_.is-editor-empty:first-child::before]:h-0 [&_.is-editor-empty:first-child::before]:text-muted-foreground [&_.is-editor-empty:first-child::before]:content-[attr(data-placeholder)]',
				)}
			/>
		</div>
	)
}
