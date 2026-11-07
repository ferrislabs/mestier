import type { FormEvent, ReactNode } from 'react'
import { Button } from '#/components/ui/button'

export function SettingsCard({
	id,
	title,
	description,
	children,
}: {
	id: string
	title: string
	description?: string
	children: ReactNode
}) {
	const titleId = `${id}-title`
	return (
		<section
			id={id}
			aria-labelledby={titleId}
			tabIndex={-1}
			className="scroll-mt-6 rounded-lg border bg-card outline-none"
		>
			<header className="border-b px-5 py-4">
				<h2 id={titleId} className="font-semibold text-base">
					{title}
				</h2>
				{description && (
					<p className="mt-1 text-[13px] text-muted-foreground">
						{description}
					</p>
				)}
			</header>
			{children}
		</section>
	)
}

export function SettingsForm({
	onSubmit,
	dirty,
	saved,
	savedLabel = 'Modifications enregistrées.',
	error = null,
	children,
}: {
	onSubmit: (event: FormEvent) => void
	dirty: boolean
	saved: boolean
	savedLabel?: string
	error?: string | null
	children: ReactNode
}) {
	const status = saved
		? savedLabel
		: dirty
			? 'Modifications non enregistrées.'
			: ''
	return (
		<form onSubmit={onSubmit} noValidate>
			<div className="grid gap-4 px-5 py-5">
				{children}
				{error && (
					<p role="alert" className="text-[13px] text-state-alert">
						{error}
					</p>
				)}
			</div>
			<div className="flex items-center justify-between gap-3 border-t px-5 py-3">
				<output className="text-[13px] text-muted-foreground">{status}</output>
				<Button type="submit" size="sm" disabled={!dirty}>
					Enregistrer
				</Button>
			</div>
		</form>
	)
}
