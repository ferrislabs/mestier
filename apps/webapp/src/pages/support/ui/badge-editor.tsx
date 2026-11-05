import { Check, Plus, X } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import {
	BADGE_MAX_LENGTH,
	normalizeBadge,
	sameBadge,
} from '#/pages/support/lib/badges'
import type { SupportBadgeVM } from '#/pages/support/types'
import { badgeToneClassName } from '#/pages/support/ui/custom-field-chip'

export function BadgeEditor({
	badges,
	suggestions,
	disabled = false,
	onAdd,
	onRemove,
}: {
	badges: readonly SupportBadgeVM[]
	suggestions: readonly string[]
	disabled?: boolean
	onAdd: (label: string) => void
	onRemove: (label: string) => void
}) {
	const [query, setQuery] = useState('')
	const label = normalizeBadge(query)
	const options = [
		...new Set([...suggestions, ...badges.map((badge) => badge.label)]),
	].sort((a, b) => a.localeCompare(b, 'fr'))
	const canCreate =
		label !== '' && !options.some((option) => sameBadge(option, label))

	return (
		<div className="flex flex-wrap items-center gap-1.5">
			{badges.map((badge) => (
				<span
					key={badge.label}
					className={cn(
						'inline-flex items-center gap-0.5 rounded-sm py-px pr-0.5 pl-1.5 font-medium text-[11px]',
						badgeToneClassName(badge.tone),
					)}
				>
					{badge.label}
					<button
						type="button"
						disabled={disabled}
						onClick={() => onRemove(badge.label)}
						aria-label={`Retirer le badge ${badge.label}`}
						className="rounded-xs p-0.5 text-primary-foreground/80 outline-none hover:text-primary-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none"
					>
						<X className="size-3" aria-hidden />
					</button>
				</span>
			))}
			<Popover onOpenChange={(open) => !open && setQuery('')}>
				<PopoverTrigger asChild>
					<Button
						variant="ghost"
						size="sm"
						disabled={disabled}
						className="h-6 px-1.5 text-muted-foreground text-xs"
					>
						<Plus />
						{badges.length === 0 ? 'Ajouter un badge' : 'Ajouter'}
					</Button>
				</PopoverTrigger>
				<PopoverContent align="start" className="w-64 p-0">
					<Command>
						<CommandInput
							value={query}
							onValueChange={setQuery}
							maxLength={BADGE_MAX_LENGTH}
							placeholder="Chercher ou créer un badge…"
						/>
						<CommandList>
							{!canCreate && (
								<CommandEmpty>Tapez un nom pour créer un badge.</CommandEmpty>
							)}
							<CommandGroup>
								{options.map((option) => {
									const selected = badges.some((badge) =>
										sameBadge(badge.label, option),
									)
									return (
										<CommandItem
											key={option}
											value={option}
											onSelect={() =>
												selected ? onRemove(option) : onAdd(option)
											}
											aria-selected={selected}
										>
											<span className="min-w-0 flex-1 truncate">{option}</span>
											<Check
												className={cn(
													'size-4',
													selected ? 'opacity-100' : 'opacity-0',
												)}
												aria-hidden
											/>
										</CommandItem>
									)
								})}
							</CommandGroup>
							{canCreate && (
								<CommandGroup forceMount>
									<CommandItem
										forceMount
										value={`créer-${label}`}
										onSelect={() => {
											onAdd(label)
											setQuery('')
										}}
									>
										<Plus aria-hidden />
										<span className="min-w-0 flex-1 truncate">
											Créer « {label} »
										</span>
									</CommandItem>
								</CommandGroup>
							)}
						</CommandList>
					</Command>
				</PopoverContent>
			</Popover>
		</div>
	)
}
