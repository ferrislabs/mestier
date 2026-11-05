import { Link } from '@tanstack/react-router'
import { ModuleIcon } from '#/components/module-icon'
import type { AppModule } from '#/modules/types'

export interface AppLauncherItem {
	id: string
	label: string
	module: Pick<AppModule, 'icon' | 'hue'>
	to: string
	hint: string
	comingSoon?: boolean
}

export interface AppLauncherUIProps {
	items: AppLauncherItem[]
}

export function AppLauncherUI({ items }: AppLauncherUIProps) {
	return (
		<nav
			aria-label="Accès rapides"
			className="mt-6 grid grid-cols-2 gap-2 sm:grid-cols-4"
		>
			{items.map((item) => {
				const content = (
					<>
						<ModuleIcon module={item.module} size="md" />
						<span className="min-w-0">
							<span className="block truncate font-medium text-[13.5px]">
								{item.label}
							</span>
							<span className="block truncate text-[12px] text-muted-foreground">
								{item.hint}
							</span>
						</span>
					</>
				)

				if (item.comingSoon) {
					return (
						<div
							key={item.id}
							aria-disabled="true"
							className="flex cursor-not-allowed items-center gap-3 rounded-lg border bg-card px-3 py-2.5 opacity-60"
						>
							{content}
						</div>
					)
				}

				return (
					<Link
						key={item.id}
						to={item.to}
						className="flex items-center gap-3 rounded-lg border bg-card px-3 py-2.5 outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
					>
						{content}
					</Link>
				)
			})}
		</nav>
	)
}
