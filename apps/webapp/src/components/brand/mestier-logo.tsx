import { cn } from '#/lib/utils'

interface MestierMarkProps {
	className?: string
}

function MestierMark({ className }: MestierMarkProps) {
	return (
		<svg
			viewBox="0 0 48 48"
			aria-hidden="true"
			className={cn('size-8 shrink-0 text-primary', className)}
			fill="none"
		>
			<path d="M7 8h8c8.284 0 15 6.716 15 15v17H7V8Z" fill="currentColor" />
			<path d="M41 8h-8c-8.284 0-15 6.716-15 15v17h23V8Z" fill="currentColor" />
			<path
				d="M24 28c2.2-5.2 5.7-8.5 10-9.9V40H24V28Z"
				fill="var(--brand-mark-cutout)"
			/>
			<path
				d="M24 28c-2.2-5.2-5.7-8.5-10-9.9V40h10V28Z"
				fill="var(--brand-mark-cutout)"
			/>
		</svg>
	)
}

interface MestierLogoProps {
	className?: string
	markClassName?: string
	textClassName?: string
	showText?: boolean
}

function MestierLogo({
	className,
	markClassName,
	textClassName,
	showText = true,
}: MestierLogoProps) {
	return (
		<div className={cn('inline-flex items-center gap-3', className)}>
			<MestierMark className={markClassName} />
			{showText ? (
				<span
					className={cn(
						'text-xl font-semibold leading-none text-primary',
						textClassName,
					)}
				>
					Mestier
				</span>
			) : null}
		</div>
	)
}

function MestierAppIcon({ className }: MestierMarkProps) {
	// `icon.svg` porte déjà son fond de marque et ses coins arrondis : le
	// conteneur ne fait que contraindre la taille.
	return (
		<div
			className={cn(
				'flex size-10 items-center justify-center overflow-hidden rounded-xl',
				className,
			)}
		>
			<img src="/icon.svg" alt="Mestier" className="size-full" />
		</div>
	)
}

export { MestierAppIcon, MestierLogo, MestierMark }
