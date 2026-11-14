import type { ReactNode } from 'react'
import { MestierMark } from '#/components/brand/mestier-logo'
import { cn } from '#/lib/utils'

export type RespondVariant = 'public' | 'member'

export function RespondLayout({
	variant,
	title,
	description,
	children,
}: {
	variant: RespondVariant
	title: string | null
	description?: string | null
	children: ReactNode
}) {
	return (
		<div
			className={cn(
				'flex w-full flex-col overflow-x-hidden bg-background text-foreground',
				variant === 'public' ? 'min-h-dvh' : 'min-h-full',
			)}
		>
			{variant === 'public' ? (
				<div
					aria-hidden
					className="h-1.5 w-full bg-gradient-to-r from-primary via-highlight to-primary"
				/>
			) : null}
			<div className="mx-auto flex w-full max-w-xl flex-1 flex-col px-4 sm:px-6">
				<header className="flex flex-col gap-2 pt-6 pb-6 sm:pt-10">
					{variant === 'public' ? (
						<MestierMark className="mb-2 size-7" />
					) : null}
					{title ? (
						<h1 className="text-sm font-semibold uppercase tracking-wide text-primary break-words">
							{title}
						</h1>
					) : null}
					{description ? (
						<p className="text-sm text-muted-foreground break-words whitespace-pre-line">
							{description}
						</p>
					) : null}
				</header>
				<main className="flex flex-1 flex-col">{children}</main>
			</div>
		</div>
	)
}
