import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { ClosureReasonsFeature } from '#/pages/cursus/catalog/feature/cursus-catalog-features'

export function CursusSettingsSection() {
	const { activeOrganization } = useActiveOrganization()
	const canView = useHasPermission('VIEW_CURSUS')

	if (!canView) {
		return (
			<p className="text-sm text-muted-foreground">
				Vous n'avez pas accès aux réglages Cursus.
			</p>
		)
	}

	return (
		<div className="flex flex-col gap-6" key={activeOrganization.id}>
			<ClosureReasonsFeature />
		</div>
	)
}
