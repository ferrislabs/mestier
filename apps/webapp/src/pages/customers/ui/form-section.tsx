import type { ReactNode } from 'react'

export function FormSection({
	label,
	hint,
	children,
}: {
	label: string
	hint?: string
	children: ReactNode
}) {
	return (
		<section aria-label={label} className="grid gap-3">
			<div>
				<h3 className="font-medium text-sm">{label}</h3>
				{hint && <p className="text-[12px] text-muted-foreground">{hint}</p>}
			</div>
			<div className="grid gap-3 sm:grid-cols-2">{children}</div>
		</section>
	)
}

export function FormErrors({ messages }: { messages: readonly string[] }) {
	if (messages.length === 0) return null
	return (
		<ul className="grid gap-1 text-[13px] text-state-alert">
			{messages.map((message) => (
				<li key={message}>{message}</li>
			))}
		</ul>
	)
}
