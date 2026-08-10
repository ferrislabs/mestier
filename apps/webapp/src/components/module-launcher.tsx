import { Link } from '@tanstack/react-router'
import { useState } from 'react'

import { Button } from '#/components/ui/button'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import { MODULES } from '#/modules/registry'
import type { AppModule, ModuleId } from '#/modules/types'

interface ModuleLauncherProps {
	activeModuleId: ModuleId
	organizationSlug: string
}

const tileClassName =
	'flex h-24 w-full flex-col items-center justify-center gap-2 rounded-xl p-2 text-center text-xs font-medium transition-colors'

const markClassName = 'flex size-11 items-center justify-center rounded-full'

/** Nine-dot grid: the app-switching affordance. */
function AppsGridIcon() {
	const positions = [5, 12, 19]

	return (
		<svg
			viewBox="0 0 24 24"
			fill="currentColor"
			aria-hidden="true"
			className="size-5"
		>
			{positions.map((cy) =>
				positions.map((cx) => (
					<circle key={`${cx}-${cy}`} cx={cx} cy={cy} r="1.9" />
				)),
			)}
		</svg>
	)
}

/**
 * App launcher, modelled on Google's app grid: one button opens the full list
 * of modules, available and announced alike.
 *
 * Open state is driven here rather than delegated to `PopoverClose`: closing is
 * triggered by the click on the link, which no longer depends on the execution
 * order between Radix's handlers and the router's.
 */
export function ModuleLauncher({
	activeModuleId,
	organizationSlug,
}: ModuleLauncherProps) {
	const [open, setOpen] = useState(false)
	const modules = MODULES.filter((module) => module.status !== 'hidden')

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button
					variant="ghost"
					size="icon"
					aria-label="Changer de module"
					className="text-muted-foreground"
				>
					<AppsGridIcon />
				</Button>
			</PopoverTrigger>
			<PopoverContent
				align="end"
				sideOffset={8}
				className="w-80 rounded-2xl p-4"
			>
				<p className="mb-3 px-1 text-xs font-medium text-muted-foreground">
					Modules Mestier
				</p>
				<ul aria-label="Modules" className="grid grid-cols-3 gap-1">
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
	const Icon = module.icon

	if (module.status !== 'available') {
		return (
			<button
				type="button"
				aria-disabled="true"
				className={cn(
					tileClassName,
					'cursor-not-allowed text-muted-foreground',
				)}
			>
				<span className={cn(markClassName, 'bg-muted text-muted-foreground')}>
					<Icon className="size-5" />
				</span>
				<span className="w-full truncate text-[11px] leading-tight">
					{module.label}
				</span>
				<span className="text-[9px] font-normal">bientôt</span>
			</button>
		)
	}

	return (
		<Link
			to={buildOrgPath(organizationSlug, module.basePath)}
			onClick={onNavigate}
			aria-current={active ? 'page' : undefined}
			className={cn(
				tileClassName,
				'hover:bg-muted',
				active ? 'text-primary' : 'text-foreground',
			)}
		>
			<span
				className={cn(
					markClassName,
					active
						? 'bg-primary text-primary-foreground'
						: 'bg-brand-soft text-primary',
				)}
			>
				<Icon className="size-5" />
			</span>
			<span className="w-full truncate text-[11px]">{module.label}</span>
		</Link>
	)
}
