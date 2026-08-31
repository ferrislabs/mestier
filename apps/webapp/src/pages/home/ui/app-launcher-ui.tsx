import { Link } from '@tanstack/react-router'
import type { LucideIcon } from 'lucide-react'

export interface AppLauncherItem {
	id: string
	label: string
	icon: LucideIcon
	to: string
}

export interface AppLauncherUIProps {
	items: AppLauncherItem[]
}

/**
 * The homepage's module shortcuts, as an icon grid rather than the text pill
 * row it used to be — every module the sidebar (`modules/registry.ts`) already
 * knows about, so a module added there shows up here for free.
 */
export function AppLauncherUI({ items }: AppLauncherUIProps) {
	return (
		<nav className="grid w-full max-w-xl grid-cols-3 gap-1 sm:grid-cols-6">
			{items.map((item) => (
				<Link
					key={item.id}
					to={item.to}
					className="flex flex-col items-center gap-2 rounded-2xl px-1 py-3 text-center transition hover:-translate-y-0.5 hover:bg-card"
				>
					<span className="flex size-10 items-center justify-center rounded-xl bg-brand-soft text-primary">
						<item.icon className="size-[1.15rem]" />
					</span>
					<span className="text-xs font-medium text-muted-foreground">
						{item.label}
					</span>
				</Link>
			))}
		</nav>
	)
}
