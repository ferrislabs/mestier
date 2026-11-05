import { useEffect, useId, useRef, useState } from 'react'
import { cn } from '#/lib/utils'

const SURFACE =
	'-mx-1.5 min-w-0 truncate rounded-sm border border-transparent bg-transparent px-1.5 py-0.5 text-left outline-none transition-colors'

export function InlineEditField({
	value,
	label,
	placeholder,
	validate,
	onCommit,
	className,
}: {
	value: string
	label: string
	placeholder?: string
	validate?: (value: string) => string | null
	onCommit: (value: string) => void
	className?: string
}) {
	const errorId = useId()
	const inputRef = useRef<HTMLInputElement>(null)
	const [editing, setEditing] = useState(false)
	const [draft, setDraft] = useState(value)
	const [error, setError] = useState<string | null>(null)

	useEffect(() => {
		if (!editing) return
		inputRef.current?.focus()
		inputRef.current?.select()
	}, [editing])

	const stop = () => {
		setDraft(value)
		setError(null)
		setEditing(false)
	}

	const commit = () => {
		const trimmed = draft.trim()
		if (trimmed === value) return stop()
		const problem = validate?.(trimmed) ?? null
		setError(problem)
		if (problem) return
		onCommit(trimmed)
		setEditing(false)
	}

	return (
		<div className="grid min-w-0">
			{editing ? (
				<input
					ref={inputRef}
					value={draft}
					placeholder={placeholder}
					aria-label={label}
					onChange={(event) => {
						setDraft(event.target.value)
						setError(null)
					}}
					onBlur={commit}
					onKeyDown={(event) => {
						if (event.key === 'Enter') {
							event.preventDefault()
							commit()
						}
						if (event.key === 'Escape') stop()
					}}
					data-pending-edit="true"
					aria-invalid={error ? true : undefined}
					aria-describedby={error ? errorId : undefined}
					className={cn(
						SURFACE,
						'border-ring bg-background ring-2 ring-ring/30 placeholder:text-muted-foreground/70 aria-invalid:border-state-alert',
						className,
					)}
				/>
			) : (
				<button
					type="button"
					onClick={() => {
						setDraft(value)
						setEditing(true)
					}}
					aria-label={`${label} : ${value || 'vide'}. Cliquer pour modifier`}
					title="Cliquer pour modifier"
					className={cn(
						SURFACE,
						'cursor-text hover:border-border focus-visible:border-ring',
						className,
						!value && 'text-muted-foreground/70',
					)}
				>
					{value || placeholder}
				</button>
			)}
			{error && (
				<p id={errorId} className="mt-0.5 text-state-alert text-xs">
					{error}
				</p>
			)}
		</div>
	)
}

export function isPendingEdit(target: EventTarget | null): boolean {
	return target instanceof HTMLElement && target.dataset.pendingEdit === 'true'
}
