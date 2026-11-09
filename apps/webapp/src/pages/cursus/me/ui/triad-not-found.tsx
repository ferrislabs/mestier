import { EmptyState } from '#/components/empty-state'
import { Button } from '#/components/ui/button'
import { PageShell } from '#/components/ui/surface'

export function TriadNotFound({ onBack }: { onBack: () => void }) {
	return (
		<PageShell>
			<EmptyState
				title="Trinôme introuvable"
				hint="Ce trinôme n’existe pas ou ne fait pas partie de vos trinômes."
			>
				<Button type="button" variant="outline" onClick={onBack}>
					Retour aux trinômes
				</Button>
			</EmptyState>
		</PageShell>
	)
}
