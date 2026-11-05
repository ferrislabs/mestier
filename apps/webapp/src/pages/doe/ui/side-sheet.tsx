import type { ReactNode } from 'react'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'

export function SideSheet({
	open,
	onOpenChange,
	title,
	description,
	children,
}: {
	open: boolean
	onOpenChange: (open: boolean) => void
	title: string
	description: string
	children: ReactNode
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="top-0 right-0 left-auto h-dvh max-w-full translate-x-0 translate-y-0 gap-0 rounded-none border-y-0 border-r-0 p-0 data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right data-[state=closed]:zoom-out-100 data-[state=open]:zoom-in-100 sm:max-w-xl">
				<div className="flex h-full min-h-0 flex-col">
					<DialogHeader className="border-b px-6 py-4 pr-12">
						<DialogTitle>{title}</DialogTitle>
						<DialogDescription>{description}</DialogDescription>
					</DialogHeader>
					{children}
				</div>
			</DialogContent>
		</Dialog>
	)
}
