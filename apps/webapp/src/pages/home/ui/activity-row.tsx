import { Link } from '@tanstack/react-router'
import { ModuleIcon } from '#/components/module-icon'
import { cn } from '#/lib/utils'
import type { HomeActivity } from '#/pages/home/types'

export function ActivityRow({
	item,
	showModule = true,
	className,
}: {
	item: HomeActivity
	showModule?: boolean
	className?: string
}) {
	return (
		<Link
			to={item.to}
			className={cn(
				'group flex items-start gap-3 px-4 py-3 outline-none transition-colors hover:bg-accent/60 focus-visible:bg-accent',
				className,
			)}
		>
			<span className="min-w-0 flex-1">
				<span className="block truncate font-medium text-[13.5px]">
					{item.title}
				</span>
				{item.context && (
					<span className="block truncate text-[12.5px] text-muted-foreground">
						{item.context}
					</span>
				)}
				{item.progress !== undefined && (
					<span className="mt-2 flex items-center gap-2">
						<span className="h-1 max-w-48 flex-1 overflow-hidden rounded-full bg-muted">
							<span
								className="block h-full rounded-full bg-state-progress"
								style={{ width: `${item.progress}%` }}
							/>
						</span>
						<span className="tabular text-[11px] text-muted-foreground">
							{item.progress} %
						</span>
					</span>
				)}
			</span>
			<span className="flex shrink-0 flex-col items-end gap-1 text-right">
				{item.when && (
					<span className="text-[12px] text-muted-foreground">{item.when}</span>
				)}
				{showModule && (
					<span className="flex items-center gap-1.5 text-[12px] text-muted-foreground">
						<ModuleIcon
							module={item.module}
							glyph
							size="sm"
							className="size-4 [&_svg]:size-3.5"
						/>
						{item.module.label}
					</span>
				)}
			</span>
		</Link>
	)
}
