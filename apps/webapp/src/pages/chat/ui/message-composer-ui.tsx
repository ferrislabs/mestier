import {
	AtSign,
	Loader2,
	Paperclip,
	SendHorizontal,
	Smile,
	X,
} from 'lucide-react'
import { useLayoutEffect, useRef } from 'react'
import { Button } from '#/components/ui/button'
import { Kbd } from '#/components/ui/kbd'
import { cn } from '#/lib/utils'
import { EmojiPopover } from '#/pages/chat/ui/emoji-popover'
import { IconButton } from '#/pages/chat/ui/icon-button'

const MAX_TEXTAREA_HEIGHT_PX = 160

export interface PendingAttachment {
	id: string
	filename: string
	status: 'uploading' | 'ready' | 'failed'
}

export interface MessageComposerUIProps {
	value: string
	onChange: (value: string) => void
	onSend: () => void
	onTyping: () => void
	pendingAttachments: PendingAttachment[]
	onAttachFiles: (files: FileList) => void
	onRemoveAttachment: (id: string) => void
	label?: string
	placeholder?: string
}

export function MessageComposerUI({
	value,
	onChange,
	onSend,
	onTyping,
	pendingAttachments,
	onAttachFiles,
	onRemoveAttachment,
	label = 'Message',
	placeholder = 'Écrire un message…',
}: MessageComposerUIProps) {
	const textareaRef = useRef<HTMLTextAreaElement>(null)
	const fileInputRef = useRef<HTMLInputElement>(null)
	const hasBlockingUpload = pendingAttachments.some(
		(attachment) => attachment.status === 'uploading',
	)
	const canSend =
		(value.trim().length > 0 || pendingAttachments.length > 0) &&
		!hasBlockingUpload

	useLayoutEffect(() => {
		const textarea = textareaRef.current
		if (!textarea) return
		textarea.style.height = 'auto'
		if (value !== '')
			textarea.style.height = `${Math.min(textarea.scrollHeight, MAX_TEXTAREA_HEIGHT_PX)}px`
	}, [value])

	function submit() {
		if (!canSend) return
		onSend()
	}

	function appendMention() {
		onChange(value === '' || value.endsWith(' ') ? `${value}@` : `${value} @`)
		textareaRef.current?.focus()
	}

	function appendEmoji(emoji: string) {
		onChange(`${value}${emoji}`)
		textareaRef.current?.focus()
	}

	return (
		<form
			aria-label="Nouveau message"
			onSubmit={(event) => {
				event.preventDefault()
				submit()
			}}
			className="shrink-0 bg-background px-4 pt-1 pb-4"
		>
			{pendingAttachments.length > 0 ? (
				<ul className="mb-2 flex flex-wrap gap-2">
					{pendingAttachments.map((attachment) => (
						<li
							key={attachment.id}
							className={cn(
								'flex items-center gap-1.5 rounded-md border bg-card px-2 py-1 text-xs',
								attachment.status === 'failed' &&
									'border-destructive text-destructive',
							)}
						>
							{attachment.status === 'uploading' ? (
								<Loader2 className="size-3 animate-spin" />
							) : (
								<Paperclip className="size-3" />
							)}
							<span className="max-w-40 truncate">{attachment.filename}</span>
							<button
								type="button"
								onClick={() => onRemoveAttachment(attachment.id)}
								aria-label={`Retirer ${attachment.filename}`}
							>
								<X className="size-3" />
							</button>
						</li>
					))}
				</ul>
			) : null}
			<div className="rounded-lg border border-input bg-card transition-shadow focus-within:border-ring focus-within:ring-[3px] focus-within:ring-ring/50">
				<textarea
					ref={textareaRef}
					rows={1}
					value={value}
					aria-label={label}
					placeholder={placeholder}
					onChange={(event) => {
						onChange(event.target.value)
						onTyping()
					}}
					onKeyDown={(event) => {
						if (
							event.key !== 'Enter' ||
							event.shiftKey ||
							event.nativeEvent.isComposing
						)
							return
						event.preventDefault()
						submit()
					}}
					className="block max-h-40 w-full resize-none overflow-y-auto bg-transparent px-3 pt-3 pb-1 text-sm leading-relaxed outline-none placeholder:text-muted-foreground"
				/>
				<div className="flex items-center gap-0.5 px-2 pb-2">
					<IconButton
						label="Joindre un fichier"
						icon={Paperclip}
						onClick={() => fileInputRef.current?.click()}
					/>
					<input
						ref={fileInputRef}
						type="file"
						multiple
						aria-label="Fichiers à joindre"
						className="hidden"
						onChange={(event) => {
							if (event.target.files && event.target.files.length > 0) {
								onAttachFiles(event.target.files)
							}
							event.target.value = ''
						}}
					/>
					<IconButton
						label="Mentionner un collègue"
						icon={AtSign}
						onClick={appendMention}
					/>
					<EmojiPopover side="top" align="start" onSelect={appendEmoji}>
						<IconButton label="Insérer un emoji" icon={Smile} />
					</EmojiPopover>
					<p className="ml-auto mr-2 hidden items-center gap-1 text-muted-foreground text-xs sm:flex">
						<Kbd>Entrée</Kbd>
						<span>envoyer</span>
						<span aria-hidden>·</span>
						<Kbd>Maj</Kbd>
						<span aria-hidden>+</span>
						<Kbd>Entrée</Kbd>
						<span>nouvelle ligne</span>
					</p>
					<Button
						type="submit"
						size="icon-sm"
						aria-label="Envoyer le message"
						disabled={!canSend}
						className="max-sm:ml-auto"
					>
						<SendHorizontal aria-hidden />
					</Button>
				</div>
			</div>
		</form>
	)
}
