import { LogOut } from 'lucide-react'
import { MestierLogo } from '#/components/brand/mestier-logo'
import { Button } from '#/components/ui/button'

interface PortalHeaderProps {
	displayName: string
	onSignOut: () => void
}

export function PortalHeader({ displayName, onSignOut }: PortalHeaderProps) {
	return (
		<header className="flex h-14 items-center justify-between border-b bg-card px-6">
			<MestierLogo markClassName="size-7" textClassName="text-lg" />
			<div className="flex items-center gap-3 text-[13px]">
				<span className="truncate">{displayName}</span>
				<Button type="button" variant="outline" size="sm" onClick={onSignOut}>
					<LogOut />
					Se déconnecter
				</Button>
			</div>
		</header>
	)
}
