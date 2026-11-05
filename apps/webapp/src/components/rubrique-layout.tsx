import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'

import { cn } from '#/lib/utils'

export interface RubriqueNavItem {
	id: string
	label: string
	summary: string
	icon: LucideIcon
}

export interface RubriqueLinkProps {
	className: string
	'aria-current': 'page' | undefined
	children: ReactNode
}

interface RubriqueNavProps {
	label: string
	items: readonly RubriqueNavItem[]
	selected: string
	renderLink: (item: RubriqueNavItem, props: RubriqueLinkProps) => ReactNode
}

export function RubriqueNav({
	label,
	items,
	selected,
	renderLink,
}: RubriqueNavProps) {
	return (
		<nav
			aria-label={label}
			className="min-h-0 overflow-x-auto lg:flex-1 lg:overflow-y-auto lg:overflow-x-hidden"
		>
			<ul className="flex lg:block">
				{items.map((item) => {
					const Icon = item.icon
					const isSelected = item.id === selected
					return (
						<li key={item.id} className="shrink-0">
							{renderLink(item, {
								className: cn(
									'relative flex items-center gap-3 border-border px-4 py-3 outline-none transition-colors hover:bg-accent/60 focus-visible:bg-accent max-lg:py-2.5 lg:border-b',
									isSelected && 'bg-accent',
								),
								'aria-current': isSelected ? 'page' : undefined,
								children: (
									<>
										{isSelected && (
											<span
												aria-hidden
												className="absolute bg-primary max-lg:inset-x-0 max-lg:bottom-0 max-lg:h-0.5 lg:inset-y-0 lg:left-0 lg:w-0.5"
											/>
										)}
										<span
											aria-hidden
											className="flex size-9 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
										>
											<Icon className="size-4" />
										</span>
										<span className="grid min-w-0 flex-1 gap-0.5">
											<span className="truncate font-medium text-sm">
												{item.label}
											</span>
											<span className="truncate text-muted-foreground text-xs max-lg:hidden">
												{item.summary}
											</span>
										</span>
									</>
								),
							})}
						</li>
					)
				})}
			</ul>
		</nav>
	)
}

interface RubriqueLayoutProps {
	title: string
	description: string
	actions?: ReactNode
	wide?: boolean
	children: ReactNode
}

export function RubriqueLayout({
	title,
	description,
	actions,
	wide,
	children,
}: RubriqueLayoutProps) {
	return (
		<div className="flex min-w-0 flex-col lg:h-full lg:min-h-0">
			<header className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b bg-background px-6 py-4">
				<div className="min-w-0">
					<h2 className="font-semibold text-xl tracking-tight">{title}</h2>
					<p className="mt-0.5 text-muted-foreground text-sm">{description}</p>
				</div>
				{actions && (
					<div className="flex flex-wrap items-center gap-2">{actions}</div>
				)}
			</header>
			<div className="lg:min-h-0 lg:flex-1 lg:overflow-y-auto">
				<div
					className={cn(
						'mx-auto grid w-full gap-4 px-6 py-6',
						wide ? 'max-w-5xl' : 'max-w-3xl',
					)}
				>
					{children}
				</div>
			</div>
		</div>
	)
}
