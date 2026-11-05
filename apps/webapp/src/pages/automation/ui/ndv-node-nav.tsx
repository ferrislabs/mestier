import type { LucideIcon } from 'lucide-react'
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from '#/components/ui/tooltip'
import { cn } from '#/lib/utils'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'

export interface NdvNavItem {
	id: string
	label: string
	icon: LucideIcon
	hue: number
	branch?: string
}

export function NdvNodeNav({
	side,
	items,
	onNavigate,
}: {
	side: 'previous' | 'next'
	items: readonly NdvNavItem[]
	onNavigate: (nodeId: string) => void
}) {
	if (items.length === 0) return null
	const direction = side === 'previous' ? 'précédent' : 'suivant'
	return (
		<nav
			aria-label={side === 'previous' ? 'Nœuds précédents' : 'Nœuds suivants'}
			className={cn(
				'absolute top-1/2 z-10 flex -translate-y-1/2 flex-col gap-3',
				side === 'previous'
					? 'left-0 -translate-x-1/3'
					: 'right-0 translate-x-1/3',
			)}
		>
			{items.map((item) => {
				const description = item.branch
					? `${item.label} (branche ${item.branch})`
					: item.label
				return (
					<Tooltip key={item.id}>
						<TooltipTrigger asChild>
							<button
								type="button"
								onClick={() => onNavigate(item.id)}
								aria-label={`Ouvrir le nœud ${direction} : ${description}`}
								className="rounded-xl bg-card shadow-md outline-none transition-transform hover:scale-105 focus-visible:ring-2 focus-visible:ring-ring"
							>
								<KindIconTile
									icon={item.icon}
									hue={item.hue}
									className="size-11 rounded-xl"
								/>
							</button>
						</TooltipTrigger>
						<TooltipContent side={side === 'previous' ? 'right' : 'left'}>
							{description}
						</TooltipContent>
					</Tooltip>
				)
			})}
		</nav>
	)
}
