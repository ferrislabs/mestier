import { useNavigate } from '@tanstack/react-router'
import { Search } from 'lucide-react'
import { useEffect, useState } from 'react'

import { ModuleIcon } from '#/components/module-icon'
import {
	CommandDialog,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import { Kbd } from '#/components/ui/kbd'
import { usePermissions } from '#/hooks/use-permissions'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import { useVisibleModules } from '#/modules/use-visible-modules'

interface SearchCommandProps {
	organizationSlug: string
	className?: string
}

/** ⌘K palette to jump to a module or one of its sections. */
export function SearchCommand({
	organizationSlug,
	className,
}: SearchCommandProps) {
	const [open, setOpen] = useState(false)
	const navigate = useNavigate()
	const modules = useVisibleModules()
	const { data: permissionsData, isSuccess: permissionsLoaded } =
		usePermissions()
	const granted = permissionsData?.data.permissions ?? []

	useEffect(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			if (event.key === 'k' && (event.metaKey || event.ctrlKey)) {
				event.preventDefault()
				setOpen((current) => !current)
			}
		}
		window.addEventListener('keydown', onKeyDown)
		return () => window.removeEventListener('keydown', onKeyDown)
	}, [])

	const go = (path: string) => {
		setOpen(false)
		void navigate({ to: buildOrgPath(organizationSlug, path) })
	}

	return (
		<>
			<button
				type="button"
				onClick={() => setOpen(true)}
				aria-label="Rechercher"
				className={cn(
					'flex h-8 w-full max-w-sm items-center gap-2 rounded-md px-2.5 text-[13px] outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring',
					className,
				)}
			>
				<Search className="size-4 shrink-0" strokeWidth={1.8} />
				<span className="flex-1 text-left">Rechercher</span>
				<Kbd>⌘K</Kbd>
			</button>
			<CommandDialog
				open={open}
				onOpenChange={setOpen}
				title="Rechercher"
				description="Modules et sections"
			>
				<CommandInput placeholder="Aller à un module, une section…" />
				<CommandList>
					<CommandEmpty>Aucun résultat.</CommandEmpty>
					<CommandGroup heading="Modules">
						{modules
							.filter((module) => module.status === 'available')
							.map((module) => (
								<CommandItem
									key={module.id}
									value={module.label}
									onSelect={() => go(module.basePath)}
								>
									<ModuleIcon module={module} size="sm" />
									{module.label}
									{module.description ? (
										<span className="ml-auto text-muted-foreground text-xs">
											{module.description}
										</span>
									) : null}
								</CommandItem>
							))}
					</CommandGroup>
					<CommandGroup heading="Sections">
						{modules
							.filter((module) => module.status === 'available')
							.flatMap((module) =>
								module.sections
									.filter(
										(section) =>
											section.status !== 'coming-soon' &&
											(!section.requiredPermission ||
												(permissionsLoaded &&
													granted.includes(section.requiredPermission))),
									)
									.map((section) => {
										const Icon = section.icon ?? module.icon
										return (
											<CommandItem
												key={`${module.id}-${section.id}`}
												value={`${module.label} ${section.label}`}
												onSelect={() => go(section.to)}
											>
												<Icon className="text-muted-foreground" />
												{section.label}
												<span className="ml-auto text-muted-foreground text-xs">
													{module.label}
												</span>
											</CommandItem>
										)
									}),
							)}
					</CommandGroup>
				</CommandList>
			</CommandDialog>
		</>
	)
}
