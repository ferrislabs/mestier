import { KeyRound } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'
import { TextArea } from '#/pages/automation/ui/text-area'

export function SecretField({
	id,
	stored,
	multiline,
	value,
	placeholder,
	invalid,
	describedBy,
	onChange,
}: {
	id: string
	stored: boolean
	multiline: boolean
	value: string
	placeholder?: string
	invalid: boolean
	describedBy?: string
	onChange: (value: string) => void
}) {
	const [replacing, setReplacing] = useState(false)
	const inputRef = useRef<HTMLInputElement & HTMLTextAreaElement>(null)

	useEffect(() => {
		if (replacing) inputRef.current?.focus()
	}, [replacing])

	if (stored && !replacing)
		return (
			<div className="flex items-center gap-2 rounded-md border border-border border-dashed bg-muted/40 px-3 py-1.5">
				<KeyRound
					className="size-4 shrink-0 text-muted-foreground"
					aria-hidden
				/>
				<span id={id} className="flex-1 text-muted-foreground text-sm">
					Enregistré et chiffré — non affichable
				</span>
				<Button
					type="button"
					variant="outline"
					size="sm"
					onClick={() => setReplacing(true)}
				>
					Remplacer
				</Button>
			</div>
		)

	const shared = {
		id,
		value,
		placeholder: stored ? 'Nouvelle valeur' : placeholder,
		autoComplete: 'new-password',
		spellCheck: false,
		'aria-invalid': invalid || undefined,
		'aria-describedby': describedBy,
	}
	return (
		<div className="grid gap-1.5">
			{multiline ? (
				<TextArea
					{...shared}
					ref={inputRef}
					rows={4}
					onChange={(event) => onChange(event.target.value)}
					className={cn('font-mono text-xs [-webkit-text-security:disc]')}
				/>
			) : (
				<Input
					{...shared}
					ref={inputRef}
					type="password"
					onChange={(event) => onChange(event.target.value)}
				/>
			)}
			{stored && (
				<button
					type="button"
					onClick={() => {
						onChange('')
						setReplacing(false)
					}}
					className="justify-self-start rounded-sm text-muted-foreground text-xs underline-offset-2 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
				>
					Garder la valeur enregistrée
				</button>
			)}
		</div>
	)
}
