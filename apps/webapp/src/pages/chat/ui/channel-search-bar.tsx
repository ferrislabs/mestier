import { Search, X } from 'lucide-react'
import { type KeyboardEvent, useEffect, useRef } from 'react'
import { Input } from '#/components/ui/input'
import { IconButton } from '#/pages/chat/ui/icon-button'

export function ChannelSearchBar({
	query,
	label,
	resultLabel,
	onQueryChange,
	onClose,
}: {
	query: string
	label: string
	resultLabel: string
	onQueryChange: (query: string) => void
	onClose: () => void
}) {
	const inputRef = useRef<HTMLInputElement>(null)

	useEffect(() => {
		inputRef.current?.focus()
	}, [])

	function handleKeyDown(event: KeyboardEvent<HTMLInputElement>) {
		if (event.key === 'Escape') onClose()
	}

	return (
		<search className="flex shrink-0 items-center gap-2 border-border border-b bg-card px-4 py-2">
			<div className="relative flex-1">
				<Search
					aria-hidden
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
				/>
				<Input
					ref={inputRef}
					type="search"
					value={query}
					aria-label={label}
					placeholder="Rechercher un message ou un fichier"
					onChange={(event) => onQueryChange(event.target.value)}
					onKeyDown={handleKeyDown}
					className="h-8 pl-8 text-sm [&::-webkit-search-cancel-button]:hidden"
				/>
			</div>
			<p aria-live="polite" className="shrink-0 text-muted-foreground text-xs">
				{resultLabel}
			</p>
			<IconButton label="Fermer la recherche" icon={X} onClick={onClose} />
		</search>
	)
}
