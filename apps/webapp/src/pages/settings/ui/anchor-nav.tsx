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
			// top-20 (80px) must match SETTINGS_HEADER_OFFSET_PX in
			// #/pages/settings/use-active-section.ts and settings-layout.tsx's
			// scroll-mt-20 — all three move together with the header height.
			className="sticky top-20 hidden w-56 shrink-0 flex-col gap-6 self-start lg:flex"
		>
			{groups.map((group) => {
				const labelId = `settings-nav-group-${group.label.toLowerCase().replace(/\s+/g, '-')}`
				return (
					<div key={group.label} className="flex flex-col gap-1">
						<p
							id={labelId}
							className="px-3 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground"
						>
							{group.label}
						</p>
						<ul aria-labelledby={labelId} className="flex flex-col gap-1">
							{group.sections.map((section) => {
								const Icon = section.icon
								const active = section.id === activeId
								return (
									<li key={section.id}>
										<a
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
