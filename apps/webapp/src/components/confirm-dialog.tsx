import type { ReactNode } from 'react'

import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'

interface ConfirmDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	title: string
	description: string
	confirmLabel: string
	destructive?: boolean
	disabled?: boolean
	onConfirm: () => void
	children?: ReactNode
}

export function ConfirmDialog({
	open,
	onOpenChange,
	title,
	description,
	confirmLabel,
	destructive,
	disabled,
	onConfirm,
	children,
}: ConfirmDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription>{description}</DialogDescription>
				</DialogHeader>
				{children}
				<DialogFooter>
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Annuler
					</Button>
					<Button
						variant={destructive ? 'destructive' : 'default'}
						disabled={disabled}
						onClick={onConfirm}
					>
						{confirmLabel}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
