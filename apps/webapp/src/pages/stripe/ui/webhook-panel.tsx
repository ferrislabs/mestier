import { Link } from '@tanstack/react-router'
import { Copy, ShieldAlert, ShieldCheck } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { buildOrgPath } from '#/modules/org-path'

export function WebhookPanel({
	organizationSlug,
	endpoint,
	secretSet,
	onNotice,
}: {
	organizationSlug: string
	endpoint: string
	secretSet: boolean
	onNotice: (message: string) => void
}) {
	async function copy() {
		try {
			await navigator.clipboard.writeText(endpoint)
			onNotice('Adresse du webhook copiée.')
		} catch {
			onNotice('La copie a échoué : sélectionne l’adresse pour la copier.')
		}
	}

	return (
		<section
			aria-label="Webhook Stripe"
			className="mb-6 grid gap-3 rounded-lg border bg-card px-4 py-3"
		>
			<p className="text-muted-foreground text-sm">
				Stripe envoie ses événements (paiements, factures, remboursements) à
				cette adresse. Chaque événement est rapproché automatiquement des
				factures quand le client et le montant sont connus.
			</p>
			<div className="flex flex-wrap items-center gap-2">
				<code className="min-w-0 flex-1 select-all overflow-x-auto whitespace-nowrap rounded-md bg-muted px-2.5 py-1.5 font-mono text-[13px]">
					{endpoint}
				</code>
				<Button type="button" variant="outline" size="sm" onClick={copy}>
					<Copy aria-hidden />
					Copier
				</Button>
			</div>
			<div className="flex flex-wrap items-center justify-between gap-2 text-sm">
				{secretSet ? (
					<p className="flex items-center gap-1.5 text-state-progress">
						<ShieldCheck className="size-4" aria-hidden />
						Secret de signature configuré
					</p>
				) : (
					<p className="flex items-center gap-1.5 text-state-alert">
						<ShieldAlert className="size-4" aria-hidden />
						Secret de signature manquant : les événements ne peuvent pas être
						vérifiés
					</p>
				)}
				<Link
					to={buildOrgPath(organizationSlug, '/crm/reglages')}
					className="rounded-sm text-primary underline-offset-4 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
				>
					Gérer le webhook dans les réglages
				</Link>
			</div>
		</section>
	)
}
