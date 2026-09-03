import { cn } from '#/lib/utils'

interface PaperOptionListOption {
	value: string
	label: string
	description?: string
}

interface PaperOptionListProps {
	ariaLabel: string
	value: string
	options: PaperOptionListOption[]
	onChange: (value: string) => void
	emptyLabel?: string
	disabled?: boolean
}

/**
 * A plain, inline option list — deliberately not Radix's `Select`.
 *
 * `Select`'s dropdown renders in its own popper, portaled outside whatever
 * contains it; nested one level inside `EditablePaperField`'s own popover,
 * that meant a click either read as "outside" and closed the editor before
 * the value committed, or (once that was patched) never reliably reopened
 * the dropdown at all — the two Radix `Popover`/`Select` layers fighting
 * over the same pointerdown. Two rounds of patching `DismissableLayer`
 * behavior later, the simpler fix is to not nest a second popper inside the
 * first: this renders every option directly in the flow of the popover
 * that already exists, so there is only ever one layer to dismiss.
 */
export function PaperOptionList({
	ariaLabel,
	value,
	options,
	onChange,
	emptyLabel = 'Aucune option disponible',
	disabled,
}: PaperOptionListProps) {
	if (options.length === 0) {
		return (
			<p className="border px-3 py-2 text-sm text-muted-foreground">
				{emptyLabel}
			</p>
		)
	}

	return (
		<div
			role="listbox"
			aria-label={ariaLabel}
			className="max-h-48 divide-y overflow-y-auto border"
		>
			{options.map((option) => (
				<button
					key={option.value}
					type="button"
					role="option"
					aria-selected={option.value === value}
					disabled={disabled}
					onClick={() => onChange(option.value)}
					className={cn(
						'block w-full px-3 py-2 text-left text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-50',
						option.value === value
							? 'bg-brand-soft font-medium text-primary'
							: 'hover:bg-muted/60',
					)}
				>
					<span className="block truncate">{option.label}</span>
					{option.description ? (
						<span className="block truncate text-xs text-muted-foreground">
							{option.description}
						</span>
					) : null}
				</button>
			))}
		</div>
	)
}
