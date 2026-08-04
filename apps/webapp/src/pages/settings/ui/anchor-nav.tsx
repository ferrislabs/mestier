import { cn } from '#/lib/utils'
import type { SettingsNavGroup } from '#/pages/settings/nav'

interface AnchorNavProps {
	groups: SettingsNavGroup[]
	activeId: string
}

export function AnchorNav({ groups, activeId }: AnchorNavProps) {
	return (
		<nav
			aria-label="Sections de configuration"
			className="sticky top-20 hidden w-56 shrink-0 flex-col gap-6 self-start lg:flex"
		>
			{groups.map((group) => (
				<fieldset key={group.label} className="flex flex-col gap-1">
					<legend className="px-3 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
						{group.label}
					</legend>
					{group.sections.map((section) => {
						const Icon = section.icon
						const active = section.id === activeId
						return (
							<a
								key={section.id}
								href={`#${section.id}`}
								aria-current={active ? 'location' : undefined}
								className={cn(
									'flex items-center gap-2 rounded-lg px-3 py-1.5 text-sm transition-colors',
									active
										? 'bg-muted font-medium text-foreground'
										: 'text-muted-foreground hover:bg-muted/60 hover:text-foreground',
								)}
							>
								<Icon className="size-4 shrink-0" />
								<span className="truncate">{section.label}</span>
							</a>
						)
					})}
				</fieldset>
			))}
		</nav>
	)
}
