import type { LucideIcon } from 'lucide-react'
import { X } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogTitle,
} from '#/components/ui/dialog'
import { cn } from '#/lib/utils'
import { isPendingEdit } from '#/pages/automation/ui/inline-edit-field'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'
import { type NdvNavItem, NdvNodeNav } from '#/pages/automation/ui/ndv-node-nav'

export function NodeDetailsDialog({
	open,
	onOpenChange,
	title,
	description,
	heading,
	badge,
	icon,
	hue,
	status,
	input,
	parameters,
	output,
	previous,
	next,
	onNavigate,
}: {
	open: boolean
	onOpenChange: (open: boolean) => void
	title: string
	description: string
	heading?: ReactNode
	badge?: ReactNode
	icon: LucideIcon
	hue: number
	status: ReactNode
	input: ReactNode
	parameters: ReactNode
	output: ReactNode
	previous: readonly NdvNavItem[]
	next: readonly NdvNavItem[]
	onNavigate: (nodeId: string) => void
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent
				showCloseButton={false}
				onOpenAutoFocus={(event) => {
					event.preventDefault()
					if (event.currentTarget instanceof HTMLElement)
						event.currentTarget.focus()
				}}
				onEscapeKeyDown={(event) => {
					if (isPendingEdit(event.target)) event.preventDefault()
				}}
				className="h-[calc(100dvh-2.5rem)] w-[calc(100vw-2.5rem)] max-w-none gap-0 overflow-visible p-0 sm:max-w-none"
			>
				<div className="flex h-full min-h-0 flex-col overflow-hidden rounded-[inherit]">
					<header className="flex shrink-0 items-center gap-3 border-border border-b bg-card px-4 py-3">
						<KindIconTile icon={icon} hue={hue} size="lg" />
						<div className="min-w-0 flex-1">
							<DialogTitle
								className={cn(
									'truncate text-base leading-tight',
									heading && 'sr-only',
								)}
							>
								{title}
							</DialogTitle>
							<DialogDescription
								className={cn('mt-0.5 truncate', heading && 'sr-only')}
							>
								{description}
							</DialogDescription>
							{heading}
						</div>
						{badge}
						{status}
						<DialogClose asChild>
							<Button
								variant="ghost"
								size="icon-sm"
								aria-label="Fermer les détails du nœud"
							>
								<X />
							</Button>
						</DialogClose>
					</header>
					<div className="grid min-h-0 flex-1 grid-cols-1 divide-y divide-border overflow-y-auto lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_minmax(0,1fr)] lg:divide-x lg:divide-y-0 lg:overflow-hidden">
						{input}
						{parameters}
						{output}
					</div>
				</div>
				<NdvNodeNav side="previous" items={previous} onNavigate={onNavigate} />
				<NdvNodeNav side="next" items={next} onNavigate={onNavigate} />
			</DialogContent>
		</Dialog>
	)
}
