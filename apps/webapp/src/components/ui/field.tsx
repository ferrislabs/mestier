import type * as React from 'react'
import { Label } from '#/components/ui/label'
import { cn } from '#/lib/utils'

export type FieldSize = 'default' | 'compact'

export interface FieldProps
	extends Omit<React.ComponentProps<'div'>, 'children'> {
	/**
	 * Visible label text. Also the source of a stable `id` for the label's
	 * `htmlFor`, generated the same way the quote form used to (lowercased,
	 * spaces turned into dashes) whenever `htmlFor` is not passed.
	 */
	label: string
	/**
	 * Id of the control this field labels. Omit to auto-generate one from
	 * `label`. Pass `null` when the field has no single focusable control to
	 * link to (a custom multi-part widget, for instance) — no `htmlFor` is
	 * rendered in that case.
	 */
	htmlFor?: string | null
	/**
	 * `default` renders the label at its standard size. `compact` renders it
	 * `text-xs text-muted-foreground`, for dense contexts such as a line
	 * editor.
	 */
	size?: FieldSize
	/** Shows a small dot next to the label — the "unsaved change" indicator. */
	changed?: boolean
	children: React.ReactNode
}

function slugifyLabel(label: string): string {
	return label.toLowerCase().replaceAll(/\s+/g, '-')
}

/**
 * A form field: a `Label` correctly tied to its control, plus the control
 * itself. Replaces the ~10 divergent local `Field`/`FieldBlock`
 * reimplementations that used to spread spacing, label size, and `htmlFor`
 * presence across the app (#392).
 *
 * `Field` never creates the control itself — it only wraps whatever is
 * passed as `children` (an `Input`, a `Select`, a custom widget, optionally
 * followed by a hint). The caller stays responsible for setting the same
 * `id` on that control so the label really focuses it.
 */
export function Field({
	label,
	htmlFor,
	size = 'default',
	changed = false,
	className,
	children,
	...props
}: FieldProps) {
	const id = htmlFor === null ? undefined : (htmlFor ?? slugifyLabel(label))

	return (
		<div
			className={cn(
				'flex min-w-0 flex-col',
				size === 'compact' ? 'gap-1.5' : 'gap-2',
				className,
			)}
			{...props}
		>
			<Label
				htmlFor={id}
				className={
					size === 'compact' ? 'text-xs text-muted-foreground' : undefined
				}
			>
				{label}
				{changed ? <FieldChangedDot /> : null}
			</Label>
			{children}
		</div>
	)
}

function FieldChangedDot() {
	return (
		<span
			role="img"
			aria-label="modifié"
			className="ml-1.5 inline-block size-1.5 rounded-full bg-primary align-middle"
		/>
	)
}
