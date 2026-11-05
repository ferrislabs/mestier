import type { LucideIcon } from 'lucide-react'
import type { CSSProperties } from 'react'
import { cn } from '#/lib/utils'
import '#/pages/automation/ui/automation.css'

const SIZES = {
	sm: 'size-7 [&_svg]:size-4',
	md: 'size-9 [&_svg]:size-5',
	lg: 'size-11 [&_svg]:size-6',
}

export function KindIconTile({
	icon: Icon,
	hue,
	size = 'md',
	className,
}: {
	icon: LucideIcon
	hue: number
	size?: keyof typeof SIZES
	className?: string
}) {
	return (
		<span
			className={cn(
				'node-tone inline-flex shrink-0 items-center justify-center rounded-md border border-(--tone-border) bg-(--tone-bg) text-(color:--tone-fg)',
				SIZES[size],
				className,
			)}
			style={{ '--node-hue': hue } as CSSProperties}
		>
			<Icon strokeWidth={1.8} aria-hidden />
		</span>
	)
}
