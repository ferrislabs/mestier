import type { LucideIcon } from 'lucide-react'
import type { ComponentProps } from 'react'
import { Button } from '#/components/ui/button'
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from '#/components/ui/tooltip'
import { cn } from '#/lib/utils'

type IconButtonProps = Omit<ComponentProps<typeof Button>, 'children'> & {
	label: string
	icon: LucideIcon
}

export function IconButton({
	label,
	icon: Icon,
	className,
	...props
}: IconButtonProps) {
	return (
		<Tooltip>
			<TooltipTrigger asChild>
				<Button
					type="button"
					variant="ghost"
					size="icon-sm"
					aria-label={label}
					className={cn(
						'text-muted-foreground hover:text-foreground',
						className,
					)}
					{...props}
				>
					<Icon aria-hidden />
				</Button>
			</TooltipTrigger>
			<TooltipContent>{label}</TooltipContent>
		</Tooltip>
	)
}

type IconToggleProps = Omit<
	ComponentProps<typeof Button>,
	'children' | 'onClick'
> & {
	label: string
	icon: LucideIcon
	pressed: boolean
	onPressedChange: (pressed: boolean) => void
}

export function IconToggle({
	label,
	icon,
	pressed,
	onPressedChange,
	className,
	...props
}: IconToggleProps) {
	return (
		<IconButton
			label={label}
			icon={icon}
			aria-pressed={pressed}
			onClick={() => onPressedChange(!pressed)}
			className={cn(
				'aria-pressed:bg-accent aria-pressed:text-accent-foreground',
				className,
			)}
			{...props}
		/>
	)
}
