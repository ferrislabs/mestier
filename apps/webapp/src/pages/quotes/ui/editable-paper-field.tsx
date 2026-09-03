import { Pencil } from 'lucide-react'
import { useState } from 'react'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'

interface EditablePaperFieldProps {
	/** Shown as the popover's own heading — the field's committed value is
	 * printed plainly in the document, so the label only needs to exist once
	 * editing is open. */
	label: string
	className?: string
	/** The document's printed value: plain text, no input chrome. */
	children: React.ReactNode
	/** The actual controls. Radix keeps the popover mounted through its own
	 * close animation, so this builds on every render regardless of `open` —
	 * cheap, since nothing here does real work outside of an event handler.
	 * `close` lets a control that has its own confirm step (there currently
	 * are none) close the popover itself; picking a value does not need to
	 * call it, since clicking outside a `Popover` already dismisses it. */
	renderEditor: (close: () => void) => React.ReactNode
}

/**
 * A region of the quote that reads as print until it is clicked: hovering
 * shows a pencil, clicking opens the real form control in a popover anchored
 * to that spot. The alternative — Selects and Inputs sitting directly on the
 * page — is what made the previous layout read as a settings form instead of
 * the document it is supposed to become.
 */
export function EditablePaperField({
	label,
	className,
	children,
	renderEditor,
}: EditablePaperFieldProps) {
	const [open, setOpen] = useState(false)

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<button
					type="button"
					aria-label={`Modifier : ${label}`}
					className={cn(
						'group relative block w-full rounded-none p-1.5 -m-1.5 text-left outline-none transition-colors',
						'hover:bg-muted/60 focus-visible:ring-[3px] focus-visible:ring-ring/35',
						className,
					)}
				>
					{children}
					<Pencil className="pointer-events-none absolute top-1 right-1 size-3 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100" />
				</button>
			</PopoverTrigger>
			<PopoverContent
				align="start"
				className="w-80 rounded-none p-4 shadow-none"
				onInteractOutside={(event) => {
					// The editor commonly nests a Select — its dropdown renders in
					// its own Radix popper, positioned outside this popover's own
					// DOM node. Without this, clicking an option reads as an
					// outside click and closes the popover before the Select can
					// commit the value: the pencil-and-popover affordance would
					// visibly work for a client, but picking one would silently
					// fail every time.
					const target = event.target as HTMLElement | null
					if (target?.closest('[data-radix-popper-content-wrapper]')) {
						event.preventDefault()
					}
				}}
			>
				<p className="mb-3 text-xs font-semibold tracking-wide text-muted-foreground uppercase">
					{label}
				</p>
				{renderEditor(() => setOpen(false))}
			</PopoverContent>
		</Popover>
	)
}
