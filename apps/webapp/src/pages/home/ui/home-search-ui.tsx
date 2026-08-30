import { Link, useNavigate } from '@tanstack/react-router'
import { Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'

export interface SearchResultItem {
	id: string
	label: string
	sublabel?: string
	/** Fully resolved path, org prefix included — same convention as
	 * `app-breadcrumb.tsx` for links spanning several unrelated route shapes. */
	to: string
}

export interface SearchGroup {
	label: string
	items: SearchResultItem[]
}

export interface HomeSearchUIProps {
	groups: SearchGroup[]
	placeholder?: string
}

const MAX_RESULTS_PER_GROUP = 5

/**
 * The homepage's own omnibox: one input, results grouped by entity kind,
 * filtered client-side over whatever the feature layer already had loaded
 * (customers, quotes, projects, invoices) — no dedicated search endpoint
 * exists yet, so this never fires a request of its own.
 */
export function HomeSearchUI({ groups, placeholder }: HomeSearchUIProps) {
	const navigate = useNavigate()
	const [query, setQuery] = useState('')
	const [isFocused, setIsFocused] = useState(false)
	const [highlighted, setHighlighted] = useState(0)

	const matches = useMemo(() => {
		const needle = query.trim().toLowerCase()
		if (!needle) return []

		return groups
			.map((group) => ({
				label: group.label,
				items: group.items
					.filter(
						(item) =>
							item.label.toLowerCase().includes(needle) ||
							item.sublabel?.toLowerCase().includes(needle),
					)
					.slice(0, MAX_RESULTS_PER_GROUP),
			}))
			.filter((group) => group.items.length > 0)
	}, [groups, query])

	const flatResults = matches.flatMap((group) => group.items)
	const isOpen = isFocused && query.trim().length > 0

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: keeps the dropdown open while focus moves between the input and a result link; the input itself is the interactive control
		<div
			className="relative mx-auto w-full max-w-2xl"
			onBlur={(event) => {
				if (!event.currentTarget.contains(event.relatedTarget)) {
					setIsFocused(false)
				}
			}}
		>
			<div className="relative">
				<Search className="pointer-events-none absolute left-4 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
				<Input
					value={query}
					onChange={(event) => {
						setQuery(event.target.value)
						setHighlighted(0)
					}}
					onFocus={() => setIsFocused(true)}
					onKeyDown={(event) => {
						if (flatResults.length === 0) return
						if (event.key === 'ArrowDown') {
							event.preventDefault()
							setHighlighted((index) => (index + 1) % flatResults.length)
						} else if (event.key === 'ArrowUp') {
							event.preventDefault()
							setHighlighted(
								(index) =>
									(index - 1 + flatResults.length) % flatResults.length,
							)
						} else if (event.key === 'Enter') {
							const target = flatResults[highlighted]
							if (target) {
								event.preventDefault()
								setQuery('')
								setIsFocused(false)
								navigate({ to: target.to })
							}
						} else if (event.key === 'Escape') {
							setQuery('')
						}
					}}
					placeholder={
						placeholder ??
						'Rechercher un client, un devis, un projet, une facture…'
					}
					aria-label="Recherche rapide"
					className="h-12 rounded-full pl-11 text-base shadow-sm"
				/>
			</div>

			{isOpen ? (
				<div className="absolute inset-x-0 top-full z-20 mt-2 max-h-96 overflow-y-auto rounded-xl border bg-popover py-2 text-popover-foreground shadow-lg">
					{flatResults.length === 0 ? (
						<p className="px-4 py-6 text-center text-sm text-muted-foreground">
							Aucun résultat pour « {query} »
						</p>
					) : (
						matches.map((group) => (
							<div key={group.label} className="py-1">
								<p className="px-4 py-1 text-xs font-medium text-muted-foreground uppercase tracking-wide">
									{group.label}
								</p>
								{group.items.map((item) => {
									const index = flatResults.indexOf(item)
									return (
										<Link
											key={item.id}
											to={item.to}
											onClick={() => {
												setQuery('')
												setIsFocused(false)
											}}
											className={cn(
												'flex flex-col px-4 py-2 text-sm',
												index === highlighted
													? 'bg-muted/70'
													: 'hover:bg-muted/40',
											)}
										>
											<span className="truncate font-medium">{item.label}</span>
											{item.sublabel ? (
												<span className="truncate text-xs text-muted-foreground">
													{item.sublabel}
												</span>
											) : null}
										</Link>
									)
								})}
							</div>
						))
					)}
				</div>
			) : null}
		</div>
	)
}
