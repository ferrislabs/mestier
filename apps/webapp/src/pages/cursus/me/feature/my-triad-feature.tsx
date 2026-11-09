import { PageShell } from '#/components/ui/surface'
import { isNotFoundError } from '#/lib/api-error'
import { useMyCursusTriad } from '#/pages/cursus/me/feature/use-my-cursus'
import { MyTriadView } from '#/pages/cursus/me/ui/my-triad-view'
import { TriadNotFound } from '#/pages/cursus/me/ui/triad-not-found'

interface MyTriadFeatureProps {
	organizationId: string | undefined
	triadId: string
	onBack: () => void
}

export function MyTriadFeature({
	organizationId,
	triadId,
	onBack,
}: MyTriadFeatureProps) {
	const query = useMyCursusTriad(organizationId, triadId)
	const triad = query.data?.data

	if (query.isError) {
		return isNotFoundError(query.error) ? (
			<TriadNotFound onBack={onBack} />
		) : (
			<PageShell>
				<p role="alert" className="text-state-alert">
					Impossible de charger ce trinôme.
				</p>
			</PageShell>
		)
	}
	if (!triad) {
		return (
			<PageShell>
				<p className="text-muted-foreground">Chargement…</p>
			</PageShell>
		)
	}

	return <MyTriadView triad={triad} onBack={onBack} />
}
