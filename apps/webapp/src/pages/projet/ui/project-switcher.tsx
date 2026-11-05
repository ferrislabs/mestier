import { Check, ChevronsUpDown } from 'lucide-react'
import { useState } from 'react'
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

export interface SwitcherOption {
	id: string
	name: string
	ownerName: string | null
	archived: boolean
}

export interface ProjectSwitcherProps {
	currentId: string
	currentName: string
	currentOwnerName: string | null
	options: readonly SwitcherOption[]
	onSelect: (projectId: string) => void
}

export function ProjectSwitcher({
	currentId,
	currentName,
	currentOwnerName,
	options,
	onSelect,
}: ProjectSwitcherProps) {
	const [open, setOpen] = useState(false)
	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<button
					type="button"
					aria-label="Changer de projet"
					title={currentOwnerName ?? 'Projet interne'}
					className="inline-flex min-w-0 max-w-64 items-center gap-1 rounded-sm px-1 py-0.5 font-medium outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
				>
					<span className="truncate">{currentName}</span>
					<ChevronsUpDown
						className="size-3.5 shrink-0 text-muted-foreground"
						aria-hidden
					/>
				</button>
			</PopoverTrigger>
			<PopoverContent align="start" className="w-72 p-0">
				<Command>
					<CommandInput placeholder="Rechercher un projet…" />
					<CommandList>
						<CommandEmpty>Aucun projet trouvé.</CommandEmpty>
						<CommandGroup>
							{options.map((option) => (
								<CommandItem
									key={option.id}
									value={`${option.name} ${option.ownerName ?? ''}`}
									onSelect={() => {
										setOpen(false)
										onSelect(option.id)
									}}
								>
									<span className="min-w-0 flex-1">
										<span className="block truncate">{option.name}</span>
										<span className="block truncate text-muted-foreground text-xs">
											{option.ownerName ?? 'Projet interne'}
											{option.archived ? ' · archivé' : ''}
										</span>
									</span>
									<Check
										className={cn(
											'size-4',
											option.id === currentId ? 'opacity-100' : 'opacity-0',
										)}
										aria-hidden
									/>
								</CommandItem>
							))}
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	)
}
