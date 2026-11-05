import { Search } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'

const DEBOUNCE_MS = 250

export function SearchBox({
	value,
	onChange,
	label,
	placeholder,
	className,
}: {
	value: string
	onChange: (value: string) => void
	label: string
	placeholder: string
	className?: string
}) {
	const [draft, setDraft] = useState(value)
	const committed = useRef(value)

	useEffect(() => {
		if (value !== committed.current) {
			committed.current = value
			setDraft(value)
		}
	}, [value])

	useEffect(() => {
		if (draft === committed.current) return
		const timer = setTimeout(() => {
			committed.current = draft
			onChange(draft)
		}, DEBOUNCE_MS)
		return () => clearTimeout(timer)
	}, [draft, onChange])

	return (
		<div className={cn('relative w-full max-w-xs', className)}>
			<Search
				className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
				aria-hidden
			/>
			<Input
				type="search"
				value={draft}
				onChange={(event) => setDraft(event.target.value)}
				placeholder={placeholder}
				aria-label={label}
				className="h-8 bg-card pl-8"
			/>
		</div>
	)
}
