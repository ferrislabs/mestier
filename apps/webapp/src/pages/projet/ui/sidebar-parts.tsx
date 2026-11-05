import { ChevronDown } from 'lucide-react'
import { type ReactNode, useId } from 'react'
import { cn } from '#/lib/utils'

const ITEM =
	'flex h-8 items-center gap-2.5 rounded-md px-2.5 text-[13px] text-nav-muted outline-none transition-colors hover:bg-nav-active hover:text-nav-foreground focus-visible:ring-2 focus-visible:ring-ring'
const ITEM_ACTIVE = 'bg-nav-active font-medium text-nav-active-foreground'

export function sidebarItemClassName(active: boolean): string {
	return cn(ITEM, active && ITEM_ACTIVE)
}

export function SidebarEntryContent({
	leading,
	label,
	count,
	note,
}: {
	leading: ReactNode
	label: string
	count?: number
	note?: string
}) {
	return (
		<>
			<span className="flex size-4 shrink-0 items-center justify-center">
				{leading}
			</span>
			<span className="min-w-0 flex-1 truncate" title={label}>
				{label}
			</span>
			{note !== undefined && (
				<span className="tabular shrink-0 text-[11px] text-nav-muted">
					{note}
				</span>
			)}
			{count !== undefined && (
				<span className="tabular shrink-0 text-[11px] text-nav-muted">
					{count}
				</span>
			)}
		</>
	)
}

export function SidebarSection({
	title,
	collapsed,
	onToggle,
	children,
}: {
	title: string
	collapsed: boolean
	onToggle: () => void
	children: ReactNode
}) {
	const listId = useId()
	return (
		<section aria-label={title}>
			<button
				type="button"
				onClick={onToggle}
				aria-expanded={!collapsed}
				aria-controls={listId}
				className="group flex w-full items-center gap-1 rounded-md px-2.5 py-1 text-left font-semibold text-nav-muted text-xs uppercase tracking-wide outline-none transition-colors hover:text-nav-foreground focus-visible:ring-2 focus-visible:ring-ring"
			>
				<span className="flex-1">{title}</span>
				<ChevronDown
					className={cn(
						'size-3.5 transition-transform',
						collapsed && '-rotate-90',
					)}
					aria-hidden
				/>
			</button>
			<ul
				id={listId}
				hidden={collapsed}
				className="mt-0.5 flex flex-col gap-px"
			>
				{children}
			</ul>
		</section>
	)
}
