import type { ReactNode } from 'react'

export function SettingsSection({
	id,
	title,
	description,
	actions,
	children,
}: {
	id: string
	title: string
	description?: string
	actions?: ReactNode
	children: ReactNode
}) {
	const titleId = `${id}-title`
	return (
		<section aria-labelledby={titleId} className="rounded-lg border bg-card">
			<header className="flex flex-wrap items-start justify-between gap-3 border-b px-5 py-4">
				<div className="min-w-0">
					<h2 id={titleId} className="font-semibold text-base">
						{title}
					</h2>
					{description && (
						<p className="mt-1 text-[13px] text-muted-foreground">
							{description}
						</p>
					)}
				</div>
				{actions}
			</header>
			{children}
		</section>
	)
}

export function FormGroup({
	legend,
	hint,
	children,
}: {
	legend: string
	hint?: string
	children: ReactNode
}) {
	return (
		<fieldset className="grid min-w-0 gap-4 rounded-md border px-4 pt-2 pb-4">
			<legend className="px-1 font-medium text-sm">{legend}</legend>
			{hint && (
				<p className="-mt-1 text-[12px] text-muted-foreground">{hint}</p>
			)}
			{children}
		</fieldset>
	)
}
