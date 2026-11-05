import { Link } from '@tanstack/react-router'
import { ArrowLeft, FileQuestion, Loader2 } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'

export const BACK_LINK_CLASS =
	'-ml-1 inline-flex items-center gap-1.5 rounded-sm px-1 text-[13px] text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring'

export function BackLink({ to, label }: { to: string; label: string }) {
	const { activeOrganization } = useActiveOrganization()
	return (
		<Link
			to={buildOrgPath(activeOrganization.slug, to)}
			className={BACK_LINK_CLASS}
		>
			<ArrowLeft className="size-4" aria-hidden />
			{label}
		</Link>
	)
}

export function DetailShell({
	main,
	side,
	sideLabel,
	mainLabel,
	roomy,
}: {
	main: ReactNode
	side: ReactNode
	sideLabel: string
	mainLabel?: string
	roomy?: boolean
}) {
	return (
		<div
			className={cn(
				'min-h-0 flex-1 overflow-y-auto lg:grid lg:grid-rows-[minmax(0,1fr)] lg:overflow-hidden',
				roomy
					? 'lg:grid-cols-[minmax(0,1fr)_minmax(20rem,24rem)]'
					: 'lg:grid-cols-[minmax(0,1fr)_minmax(18rem,22rem)]',
			)}
		>
			<section
				aria-label={mainLabel}
				className="min-w-0 lg:min-h-0 lg:overflow-y-auto"
			>
				<div
					className={cn(
						'mx-auto grid w-full max-w-5xl',
						roomy ? 'gap-5 px-6 py-6' : 'gap-4 px-4 py-6 sm:px-6',
					)}
				>
					{main}
				</div>
			</section>
			<aside
				aria-label={sideLabel}
				className="border-border border-t bg-card lg:min-h-0 lg:overflow-y-auto lg:border-t-0 lg:border-l"
			>
				<div className="grid content-start gap-6 p-5">{side}</div>
			</aside>
		</div>
	)
}

export function SideSection({
	title,
	children,
}: {
	title: string
	children: ReactNode
}) {
	return (
		<section className="grid content-start gap-2">
			<h2 className="font-semibold text-sm">{title}</h2>
			{children}
		</section>
	)
}

function CenteredPanel({ children }: { children: ReactNode }) {
	return (
		<div className="min-h-0 flex-1 overflow-y-auto">
			<div className="mx-auto w-full max-w-3xl px-6 py-8">
				<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					{children}
				</div>
			</div>
		</div>
	)
}

export function NotFoundState({
	title,
	hint,
	children,
}: {
	title: string
	hint: string
	children?: ReactNode
}) {
	return (
		<CenteredPanel>
			<FileQuestion
				className="mx-auto mb-3 size-8 text-muted-foreground"
				aria-hidden
			/>
			<p className="font-medium">{title}</p>
			<p className="mt-1 text-[13px] text-muted-foreground">{hint}</p>
			{children && <div className="mt-4 flex justify-center">{children}</div>}
		</CenteredPanel>
	)
}

export function DetailLoading({ label }: { label: string }) {
	return (
		<CenteredPanel>
			<output className="flex items-center justify-center gap-2 text-muted-foreground text-sm">
				<Loader2 className="size-4 animate-spin" aria-hidden />
				{label}
			</output>
		</CenteredPanel>
	)
}

export function DetailError({
	title,
	message,
	backTo,
	backLabel,
	onRetry,
}: {
	title: string
	message: string
	backTo: string
	backLabel: string
	onRetry?: () => void
}) {
	const { activeOrganization } = useActiveOrganization()
	return (
		<NotFoundState title={title} hint={message}>
			<div className="flex flex-wrap justify-center gap-2">
				<Button variant="outline" size="sm" asChild>
					<Link to={buildOrgPath(activeOrganization.slug, backTo)}>
						<ArrowLeft aria-hidden /> {backLabel}
					</Link>
				</Button>
				{onRetry && (
					<Button size="sm" onClick={onRetry}>
						Réessayer
					</Button>
				)}
			</div>
		</NotFoundState>
	)
}
