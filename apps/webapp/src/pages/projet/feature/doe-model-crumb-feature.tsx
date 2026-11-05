import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useDoeModels } from '#/hooks/use-doe-models'
import { DoeModelCrumb } from '#/pages/projet/ui/doe-model-crumb'

export function DoeModelCrumbFeature({ modelId }: { modelId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const models = useDoeModels(activeOrganization.id, true)
	const model = (models.data?.data ?? []).find((item) => item.id === modelId)

	return <DoeModelCrumb name={model?.name ?? 'Modèle'} />
}
