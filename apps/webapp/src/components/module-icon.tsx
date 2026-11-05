import type { CSSProperties } from 'react'

import { cn } from '#/lib/utils'
import type { AppModule } from '#/modules/types'

const SIZES = {
	sm: 'size-6 [&_svg]:size-3.5',
	md: 'size-8 [&_svg]:size-4',
	lg: 'size-11 [&_svg]:size-5',
	xl: 'size-14 [&_svg]:size-7',
}

const DEFAULT_HUE = 165

interface ModuleIconProps {
	module: Pick<AppModule, 'icon' | 'hue'>
	size?: keyof typeof SIZES
	/** Flat glyph tinted by the module hue, no filled tile behind it. */
	glyph?: boolean
	className?: string
}

/** A module's mark. Its colour comes from the module hue and the active theme. */
export function ModuleIcon({
	module,
	size = 'md',
	glyph = false,
	className,
}: ModuleIconProps) {
	const Icon = module.icon

	return (
		<span
			className={cn(
				'inline-flex shrink-0 items-center justify-center',
				glyph ? 'module-glyph' : 'module-icon',
				SIZES[size],
				className,
			)}
			style={{ '--hue': module.hue ?? DEFAULT_HUE } as CSSProperties}
		>
			<Icon strokeWidth={glyph ? 1.6 : 1.9} aria-hidden />
		</span>
	)
}
