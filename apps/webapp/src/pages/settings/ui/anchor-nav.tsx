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
			className="flex flex-wrap gap-x-6 gap-y-3 self-start lg:sticky lg:top-6 lg:flex-col lg:gap-5"
		>
			{groups.map((group) => {
				const labelId = `settings-nav-group-${group.label.toLowerCase().replace(/\s+/g, '-')}`
				return (
					<div key={group.label} className="flex flex-col gap-1">
						<p
							id={labelId}
							className="px-3 font-medium text-[12px] text-muted-foreground"
						>
							{group.label}
						</p>
						<ul
							aria-labelledby={labelId}
							className="flex flex-wrap gap-1 lg:grid"
						>
							{group.sections.map((section) => {
								const Icon = section.icon
								const active = section.id === activeId
								return (
									<li key={section.id}>
										<a
											href={`#${section.id}`}
											aria-current={active ? 'location' : undefined}
											className={cn(
												'flex items-center gap-2 rounded-md px-3 py-1.5 text-[13px] outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring',
												active
													? 'bg-muted font-medium text-foreground'
													: 'text-muted-foreground hover:bg-muted hover:text-foreground',
											)}
										>
											<Icon className="size-4 shrink-0" />
											<span className="truncate">{section.label}</span>
										</a>
									</li>
								)
							})}
						</ul>
					</div>
				)
			})}
		</nav>
	)
}
