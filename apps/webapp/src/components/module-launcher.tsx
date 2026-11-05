import { Link } from '@tanstack/react-router'
import { LayoutGrid } from 'lucide-react'
import { useState } from 'react'

import { ModuleIcon } from '#/components/module-icon'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { AppModule, ModuleId } from '#/modules/types'
import { useVisibleModules } from '#/modules/use-visible-modules'

interface ModuleLauncherProps {
	activeModuleId: ModuleId
	organizationSlug: string
	className?: string
}

const tileClassName =
	'flex w-full flex-col items-center gap-1.5 rounded-md px-1.5 pt-2 pb-1.5 text-center outline-none focus-visible:ring-2 focus-visible:ring-ring'

/**
 * App launcher: one button opens the grid of every module, available and
 * announced alike.
 *
 * Open state is driven here rather than delegated to `PopoverClose`: closing is
 * triggered by the click on the link, which no longer depends on the execution
 * order between Radix's handlers and the router's.
 */
export function ModuleLauncher({
	activeModuleId,
	organizationSlug,
	className,
}: ModuleLauncherProps) {
	const [open, setOpen] = useState(false)
	const modules = useVisibleModules()

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<button
					type="button"
					aria-label="Changer de module"
					className={cn(
						'inline-flex size-8 shrink-0 items-center justify-center rounded-md outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring',
						className,
					)}
				>
					<LayoutGrid className="size-[18px]" strokeWidth={1.8} />
				</button>
			</PopoverTrigger>
			<PopoverContent align="start" sideOffset={6} className="w-auto p-1.5">
				<ul
					aria-label="Modules"
					className="grid grid-cols-[repeat(4,minmax(4.5rem,auto))] gap-0.5"
				>
					{modules.map((module) => (
						<li key={module.id}>
							<ModuleTile
								module={module}
								active={module.id === activeModuleId}
								organizationSlug={organizationSlug}
								onNavigate={() => setOpen(false)}
							/>
						</li>
					))}
				</ul>
			</PopoverContent>
		</Popover>
	)
}

interface ModuleTileProps {
	module: AppModule
	active: boolean
	organizationSlug: string
	onNavigate: () => void
}

function ModuleTile({
	module,
	active,
	organizationSlug,
	onNavigate,
}: ModuleTileProps) {
	if (module.status !== 'available') {
		return (
			<button
				type="button"
				aria-disabled="true"
				className={cn(tileClassName, 'cursor-not-allowed opacity-60')}
			>
				<span className="module-tile flex size-11 items-center justify-center">
					<ModuleIcon module={module} glyph size="lg" />
				</span>
				<span className="w-full whitespace-nowrap text-[11.5px] leading-tight">
					{module.label}
				</span>
				<span className="text-[9px] text-muted-foreground">bientôt</span>
			</button>
		)
	}

	return (
		<Link
			to={buildOrgPath(organizationSlug, module.basePath)}
			onClick={onNavigate}
			aria-current={active ? 'page' : undefined}
			className={cn(tileClassName, 'hover:bg-accent', active && 'bg-accent')}
		>
			<span className="module-tile flex size-11 items-center justify-center">
				<ModuleIcon module={module} glyph size="lg" />
			</span>
			<span
				className={cn(
					'w-full whitespace-nowrap text-[11.5px] leading-tight',
					active && 'font-semibold',
				)}
			>
				{module.label}
			</span>
		</Link>
	)
}
