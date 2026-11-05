import type { LucideIcon } from 'lucide-react'
import { Button } from '#/components/ui/button'

export function IconButton({
	label,
	icon: Icon,
	disabled,
	onClick,
}: {
	label: string
	icon: LucideIcon
	disabled?: boolean
	onClick: () => void
}) {
	return (
		<Button
			type="button"
			variant="ghost"
			size="icon-sm"
			aria-label={label}
			title={label}
			disabled={disabled}
			onClick={onClick}
		>
			<Icon />
		</Button>
	)
}
