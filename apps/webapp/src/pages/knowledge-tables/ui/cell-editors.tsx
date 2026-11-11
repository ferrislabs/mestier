import { Check, X } from 'lucide-react'
import { type KeyboardEvent, type ReactNode, useRef, useState } from 'react'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import { Input } from '#/components/ui/input'
import { Popover, PopoverAnchor, PopoverContent } from '#/components/ui/popover'
import { Textarea } from '#/components/ui/textarea'
import { cn } from '#/lib/utils'
import {
	invalidCellMessage,
	parseCellText,
} from '#/pages/knowledge-tables/lib/cells'
import type {
	Choice,
	ColumnKind,
	SelectOption,
} from '#/pages/knowledge-tables/lib/types'
import { OptionChip } from '#/pages/knowledge-tables/ui/option-chip'

export type CommitMove = 'stay' | 'right' | 'left' | 'down'

function moveOf(event: KeyboardEvent): CommitMove | null {
	if (event.key === 'Tab') return event.shiftKey ? 'left' : 'right'
	if (event.key === 'Enter' && !event.shiftKey) return 'stay'
	return null
}

interface TextCellEditorProps {
	kind: ColumnKind
	name: string
	initial: string
	onCommit: (value: unknown, move: CommitMove) => void
	onCancel: () => void
}

export function TextCellEditor({
	kind,
	name,
	initial,
	onCommit,
	onCancel,
}: TextCellEditorProps) {
	const [text, setText] = useState(initial)
	const [error, setError] = useState<string | null>(null)
	const done = useRef(false)

	const commit = (move: CommitMove): boolean => {
		const parsed = parseCellText(kind, text)
		if (parsed.kind === 'invalid') {
			setError(invalidCellMessage(kind))
			return false
		}
		done.current = true
		onCommit(parsed.kind === 'value' ? parsed.value : null, move)
		return true
	}

	return (
		<div className="absolute inset-0 z-30">
			<Input
				autoFocus
				aria-label={name}
				aria-invalid={error !== null}
				type={kind === 'date' ? 'date' : 'text'}
				inputMode={
					kind === 'number' || kind === 'currency' ? 'decimal' : undefined
				}
				value={text}
				className={cn(
					'h-full rounded-none border-2 border-primary bg-background px-2 text-[13px] shadow-none focus-visible:ring-0',
					(kind === 'number' || kind === 'currency') && 'text-right',
				)}
				onFocus={(event) => {
					if (kind === 'date') return
					const end = event.currentTarget.value.length
					event.currentTarget.setSelectionRange(end, end)
				}}
				onChange={(event) => {
					setText(event.target.value)
					setError(null)
				}}
				onKeyDown={(event) => {
					event.stopPropagation()
					if (event.key === 'Escape') {
						event.preventDefault()
						done.current = true
						onCancel()
						return
					}
					const move = moveOf(event)
					if (move) {
						event.preventDefault()
						commit(move)
					}
				}}
				onBlur={() => {
					if (!done.current && !commit('stay')) {
						done.current = true
						onCancel()
					}
				}}
			/>
			{error ? (
				<p
					role="alert"
					className="absolute top-full left-0 z-40 mt-1 w-64 rounded-md border border-destructive/30 bg-popover px-2 py-1 text-destructive text-xs shadow-md"
				>
					{error}
				</p>
			) : null}
		</div>
	)
}

interface PopoverEditorProps {
	anchor: ReactNode
	onClose: () => void
	onEscape: () => void
	children: ReactNode
	className?: string
}

function PopoverEditor({
	anchor,
	onClose,
	onEscape,
	children,
	className,
}: PopoverEditorProps) {
	return (
		<Popover
			open
			onOpenChange={(open) => {
				if (!open) onClose()
			}}
		>
			<PopoverAnchor asChild>{anchor}</PopoverAnchor>
			<PopoverContent
				align="start"
				sideOffset={0}
				className={cn('w-72 p-0', className)}
				onKeyDown={(event) => event.stopPropagation()}
				onEscapeKeyDown={(event) => {
					event.preventDefault()
					onEscape()
				}}
			>
				{children}
			</PopoverContent>
		</Popover>
	)
}

interface LongTextCellEditorProps {
	anchor: ReactNode
	name: string
	initial: string
	onCommit: (value: unknown, move: CommitMove) => void
	onCancel: () => void
}

export function LongTextCellEditor({
	anchor,
	name,
	initial,
	onCommit,
	onCancel,
}: LongTextCellEditorProps) {
	const [text, setText] = useState(initial)
	const [error, setError] = useState<string | null>(null)
	const done = useRef(false)

	const commit = (move: CommitMove) => {
		const parsed = parseCellText('long_text', text)
		if (parsed.kind === 'invalid') {
			setError(invalidCellMessage('long_text'))
			return
		}
		done.current = true
		onCommit(parsed.kind === 'value' ? parsed.value : null, move)
	}

	return (
		<PopoverEditor
			anchor={anchor}
			className="w-96 p-2"
			onClose={() => {
				if (!done.current) commit('stay')
			}}
			onEscape={() => {
				done.current = true
				onCancel()
			}}
		>
			<Textarea
				autoFocus
				aria-label={name}
				value={text}
				rows={8}
				className="text-[13px]"
				onChange={(event) => {
					setText(event.target.value)
					setError(null)
				}}
				onKeyDown={(event) => {
					const move = moveOf(event)
					if (move) {
						event.preventDefault()
						commit(move)
					}
				}}
			/>
			<p className="mt-1.5 text-muted-foreground text-xs">
				Entrée pour enregistrer, Maj+Entrée pour aller à la ligne, Échap pour
				annuler.
			</p>
			{error ? (
				<p role="alert" className="mt-1 text-destructive text-xs">
					{error}
				</p>
			) : null}
		</PopoverEditor>
	)
}

interface ChoiceCellEditorProps {
	anchor: ReactNode
	name: string
	options: readonly SelectOption[]
	selected: readonly string[]
	multiple: boolean
	onChange: (ids: string[]) => void
	onClose: () => void
}

export function ChoiceCellEditor({
	anchor,
	name,
	options,
	selected,
	multiple,
	onChange,
	onClose,
}: ChoiceCellEditorProps) {
	return (
		<PopoverEditor anchor={anchor} onClose={onClose} onEscape={onClose}>
			<Command>
				<CommandInput placeholder={`Chercher dans ${name}`} autoFocus />
				<CommandList>
					<CommandEmpty>Aucune option.</CommandEmpty>
					<CommandGroup>
						{options.map((option) => {
							const isOn = selected.includes(option.id)
							return (
								<CommandItem
									key={option.id}
									value={`${option.label} ${option.id}`}
									onSelect={() => {
										if (multiple) {
											onChange(
												isOn
													? selected.filter((id) => id !== option.id)
													: [...selected, option.id],
											)
										} else {
											onChange(isOn ? [] : [option.id])
											onClose()
										}
									}}
								>
									<Check
										aria-hidden
										className={cn('size-4', isOn ? 'opacity-100' : 'opacity-0')}
									/>
									<OptionChip label={option.label} color={option.color} />
								</CommandItem>
							)
						})}
						{selected.length > 0 ? (
							<CommandItem
								value="__vider"
								onSelect={() => {
									onChange([])
									onClose()
								}}
							>
								<X aria-hidden className="size-4" />
								Vider la cellule
							</CommandItem>
						) : null}
					</CommandGroup>
				</CommandList>
			</Command>
		</PopoverEditor>
	)
}

interface ReferenceCellEditorProps {
	anchor: ReactNode
	name: string
	choices: readonly Choice[]
	selectedId: string | null
	onSelect: (id: string | null) => void
	onClose: () => void
}

export function ReferenceCellEditor({
	anchor,
	name,
	choices,
	selectedId,
	onSelect,
	onClose,
}: ReferenceCellEditorProps) {
	return (
		<PopoverEditor anchor={anchor} onClose={onClose} onEscape={onClose}>
			<Command>
				<CommandInput placeholder={`Chercher dans ${name}`} autoFocus />
				<CommandList>
					<CommandEmpty>Aucun résultat.</CommandEmpty>
					<CommandGroup>
						{choices.map((choice) => (
							<CommandItem
								key={choice.id}
								value={`${choice.label} ${choice.id}`}
								onSelect={() => {
									onSelect(choice.id === selectedId ? null : choice.id)
									onClose()
								}}
							>
								<Check
									aria-hidden
									className={cn(
										'size-4',
										choice.id === selectedId ? 'opacity-100' : 'opacity-0',
									)}
								/>
								<span className="truncate">{choice.label}</span>
							</CommandItem>
						))}
						{selectedId ? (
							<CommandItem
								value="__vider"
								onSelect={() => {
									onSelect(null)
									onClose()
								}}
							>
								<X aria-hidden className="size-4" />
								Vider la cellule
							</CommandItem>
						) : null}
					</CommandGroup>
				</CommandList>
			</Command>
		</PopoverEditor>
	)
}
