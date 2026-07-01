import { Link } from '@tanstack/react-router'
import { Checkbox as CheckboxPrimitive } from 'radix-ui'
import { cn } from '#/lib/utils'

interface LegalMentionSelectorProps {
	templates: { id: string; name: string; body: string }[]
	selectedIds: string[]
	onChange: (ids: string[]) => void
	isLoading?: boolean
}

export function LegalMentionSelector({
	templates,
	selectedIds,
	onChange,
	isLoading,
}: LegalMentionSelectorProps) {
	if (isLoading) {
		return (
			<p className="text-sm text-muted-foreground">
				Chargement des mentions légales…
			</p>
		)
	}

	if (templates.length === 0) {
		return (
			<p className="text-sm text-muted-foreground">
				Aucune mention légale. Créez-en dans{' '}
				<Link
					to="/settings"
					className="font-medium text-primary underline underline-offset-2"
				>
					Paramètres → Facturation
				</Link>
				.
			</p>
		)
	}

	const toggle = (id: string) => {
		if (selectedIds.includes(id)) {
			onChange(selectedIds.filter((selectedId) => selectedId !== id))
		} else {
			onChange([...selectedIds, id])
		}
	}

	return (
		<div className="flex flex-col gap-2">
			{templates.map((template) => {
				const checked = selectedIds.includes(template.id)
				const bodyPreview =
					template.body.length > 80
						? `${template.body.slice(0, 80)}…`
						: template.body

				return (
					// biome-ignore lint/a11y/noLabelWithoutControl: CheckboxPrimitive.Root renders role="checkbox" — control is present
					<label
						key={template.id}
						className={cn(
							'flex cursor-pointer items-start gap-3 rounded-lg border bg-card px-3 py-2.5 transition-colors hover:bg-muted/40',
							checked && 'border-primary/30 bg-primary/5',
						)}
					>
						<CheckboxPrimitive.Root
							checked={checked}
							onCheckedChange={() => toggle(template.id)}
							className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded border border-input shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 data-[state=checked]:border-primary data-[state=checked]:bg-primary"
						>
							<CheckboxPrimitive.Indicator>
								<svg
									className="size-3 text-primary-foreground"
									viewBox="0 0 12 12"
									fill="none"
									xmlns="http://www.w3.org/2000/svg"
									aria-hidden="true"
								>
									<path
										d="M2 6l3 3 5-5"
										stroke="currentColor"
										strokeWidth="1.5"
										strokeLinecap="round"
										strokeLinejoin="round"
									/>
								</svg>
							</CheckboxPrimitive.Indicator>
						</CheckboxPrimitive.Root>
						<div className="min-w-0">
							<p className="text-sm font-medium leading-none">
								{template.name}
							</p>
							{bodyPreview ? (
								<p className="mt-1 truncate text-xs text-muted-foreground">
									{bodyPreview}
								</p>
							) : null}
						</div>
					</label>
				)
			})}
		</div>
	)
}
