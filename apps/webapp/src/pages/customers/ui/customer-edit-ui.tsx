import { Link } from '@tanstack/react-router'
import { AlertCircle, ArrowLeft, UserRoundX } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'
import { BackLink } from '#/pages/customers/ui/customer-sidebar'

export function CustomerEditUI({
	sidebar,
	children,
}: {
	sidebar: ReactNode
	children: ReactNode
}) {
	return (
		<div className="h-full min-h-0 flex-1 overflow-y-auto lg:grid lg:grid-cols-[minmax(20rem,24rem)_minmax(0,1fr)] lg:grid-rows-[minmax(0,1fr)] lg:overflow-hidden">
			{sidebar}
			<section aria-label="Détail du client" className="min-w-0 lg:min-h-0">
				{children}
			</section>
		</div>
	)
}

function Frame({ children }: { children: ReactNode }) {
	return (
		<div className="h-full min-h-0 flex-1 overflow-y-auto">
			<div className="mx-auto w-full max-w-3xl px-6 py-8">{children}</div>
		</div>
	)
}

export namespace CustomerEditUI {
	export function Loading({ organizationSlug }: { organizationSlug: string }) {
		return (
			<Frame>
				<BackLink organizationSlug={organizationSlug} />
				<div className="mt-4 rounded-lg border border-dashed bg-card px-6 py-12 text-center text-[13px] text-muted-foreground">
					Chargement…
				</div>
			</Frame>
		)
	}

	export function NotFound({ organizationSlug }: { organizationSlug: string }) {
		return (
			<Frame>
				<BackLink organizationSlug={organizationSlug} />
				<div className="mt-4 rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<UserRoundX
						className="mx-auto mb-3 size-8 text-muted-foreground"
						aria-hidden
					/>
					<h1 className="font-medium">Ce client est introuvable.</h1>
					<p className="mt-1 text-[13px] text-muted-foreground">
						Il n’existe pas ou l’adresse n’est plus valide.
					</p>
					<Button variant="outline" size="sm" className="mt-4" asChild>
						<Link to={buildOrgPath(organizationSlug, '/crm/customers')}>
							<ArrowLeft aria-hidden /> Retour aux clients
						</Link>
					</Button>
				</div>
			</Frame>
		)
	}

	export function LoadError({
		organizationSlug,
		message,
		onRetry,
	}: {
		organizationSlug: string
		message: string
		onRetry: () => void
	}) {
		return (
			<Frame>
				<BackLink organizationSlug={organizationSlug} />
				<div className="mt-4 rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<AlertCircle
						className="mx-auto mb-3 size-8 text-state-alert"
						aria-hidden
					/>
					<h1 className="font-medium">Impossible de charger le client.</h1>
					<p className="mt-1 text-[13px] text-muted-foreground">{message}</p>
					<Button
						variant="outline"
						size="sm"
						className="mt-4"
						onClick={onRetry}
					>
						Réessayer
					</Button>
				</div>
			</Frame>
		)
	}
}
