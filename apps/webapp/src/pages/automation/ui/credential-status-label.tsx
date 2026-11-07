import { CircleCheck, PlugZap, TriangleAlert } from 'lucide-react'
import type { CredentialStatus } from '#/hooks/use-automation'
import { cn } from '#/lib/utils'
import '#/pages/automation/ui/automation.css'

const VIEWS = {
	valid: {
		label: 'Valide',
		icon: CircleCheck,
		className: 'text-(color:--auto-success-text)',
	},
	expired: {
		label: 'Expirée',
		icon: TriangleAlert,
		className: 'text-state-alert',
	},
	not_connected: {
		label: 'Compte non connecté',
		icon: PlugZap,
		className: 'text-state-today',
	},
}

export function CredentialStatusLabel({
	status,
	className,
}: {
	status: CredentialStatus
	className?: string
}) {
	const view = VIEWS[status]
	const Icon = view.icon
	return (
		<span
			className={cn(
				'inline-flex items-center gap-1.5 font-medium text-sm',
				view.className,
				className,
			)}
		>
			<Icon className="size-4 shrink-0" aria-hidden />
			{view.label}
		</span>
	)
}
