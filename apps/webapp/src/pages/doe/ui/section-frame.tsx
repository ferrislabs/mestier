import { type ReactNode, useId } from 'react'

export function sectionHeading(number: number | null, title: string): string {
	return number === null ? title : `${number}. ${title}`
}

export function SectionFrame({
	number,
	title,
	actions,
	children,
}: {
	number: number | null
	title: string
	actions?: ReactNode
	children: ReactNode
}) {
	const headingId = useId()
	return (
		<section
			aria-labelledby={headingId}
			className="rounded-lg border bg-card text-card-foreground"
		>
			<header className="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-3">
				<h2 id={headingId} className="font-semibold text-base">
					{sectionHeading(number, title)}
				</h2>
				{actions && (
					<div className="flex flex-wrap items-center gap-2">{actions}</div>
				)}
			</header>
			{number === null && (
				<p className="border-b bg-muted/40 px-5 py-2 text-[13px] text-muted-foreground">
					Ce thème est désactivé : il n’apparaît pas dans le DOE.
				</p>
			)}
			{children}
		</section>
	)
}
