import { Link } from '@tanstack/react-router'
import { LayoutGrid } from 'lucide-react'

import {
	Popover,
	PopoverClose,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import { MODULES } from '#/modules/registry'
import type { AppModule, ModuleId } from '#/modules/types'

interface ModuleSwitcherProps {
	activeModuleId: ModuleId
}

export function ModuleSwitcher({ activeModuleId }: ModuleSwitcherProps) {
	return (
		<Popover>
			<PopoverTrigger asChild>
				<button
					type="button"
					aria-label="Changer de module"
					className="flex size-8 shrink-0 items-center justify-center rounded-lg text-white/70 transition-colors hover:bg-white/10 hover:text-white group-data-[collapsible=icon]:hidden"
				>
					<LayoutGrid className="size-4" />
				</button>
			</PopoverTrigger>
			<PopoverContent
				align="start"
				side="right"
				sideOffset={8}
				className="w-64"
			>
				<p className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
					Modules
				</p>
				<div className="grid grid-cols-3 gap-2">
					{MODULES.map((module) => (
						<ModuleTile
							key={module.id}
							module={module}
							active={module.id === activeModuleId}
						/>
					))}
				</div>
			</PopoverContent>
		</Popover>
	)
}

interface ModuleTileProps {
	module: AppModule
	active: boolean
}

function ModuleTile({ module, active }: ModuleTileProps) {
	const Icon = module.icon
	const tileClassName =
		'flex flex-col items-center gap-1.5 rounded-lg border p-2 text-center text-[11px] font-medium'

	if (!module.enabled) {
		return (
			<div
				className={cn(
					tileClassName,
					'cursor-not-allowed border-transparent text-muted-foreground opacity-60',
				)}
			>
				<Icon className="size-5" />
				<span className="truncate">{module.label}</span>
				<span className="rounded-md border px-1 py-0.5 text-[9px]">soon</span>
			</div>
		)
	}

	// Invariant : ni le onClick de fermeture (Radix) ni le handleClick de navigation
	// (TanStack Router) n'appellent preventDefault/stopPropagation — c'est cette
	// non-interférence, pas un ordre garanti, qui permet aux deux de s'exécuter ;
	// une évolution de l'une des deux libs pourrait avaler la navigation en silence.
	return (
		<PopoverClose asChild>
			<Link
				to={module.basePath}
				aria-current={active ? 'page' : undefined}
				className={cn(
					tileClassName,
					'transition-colors hover:bg-muted',
					active ? 'border-primary bg-muted' : 'border-transparent',
				)}
			>
				<Icon className="size-5" />
				<span className="truncate">{module.label}</span>
			</Link>
		</PopoverClose>
	)
}
