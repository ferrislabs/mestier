import { useState } from 'react'
import { useCreateDoe } from '#/hooks/use-doe'
import {
	useCreateDoeModelFromStarter,
	useDoeModels,
} from '#/hooks/use-doe-models'
import { mutationErrorMessage } from '#/lib/api-error'
import { activeModels, chooseCreationMode } from '#/pages/doe/lib/empty-state'
import { DoeEmptyState } from '#/pages/doe/ui/doe-empty-state'

interface DoeCreateFeatureProps {
	organizationId: string
	organizationSlug: string
	projectId: string
	hasCustomer: boolean
}

export function DoeCreateFeature({
	organizationId,
	organizationSlug,
	projectId,
	hasCustomer,
}: DoeCreateFeatureProps) {
	const models = useDoeModels(organizationId, false)
	const createDoe = useCreateDoe()
	const fromStarter = useCreateDoeModelFromStarter()
	const [chosenId, setChosenId] = useState('')

	const rows = activeModels(models.data?.data ?? [])
	const selectedModelId = rows.some((model) => model.id === chosenId)
		? chosenId
		: (rows[0]?.id ?? '')

	return (
		<DoeEmptyState
			organizationSlug={organizationSlug}
			hasCustomer={hasCustomer}
			mode={chooseCreationMode(rows)}
			models={rows}
			selectedModelId={selectedModelId}
			isLoading={models.isLoading}
			isPending={createDoe.isPending || fromStarter.isPending}
			error={
				mutationErrorMessage(createDoe.error) ??
				mutationErrorMessage(fromStarter.error) ??
				mutationErrorMessage(models.error)
			}
			onSelectModel={setChosenId}
			onCreate={() => {
				void createDoe.mutateAsync({
					path: { organization_id: organizationId, project_id: projectId },
					body: { model_id: selectedModelId },
				})
			}}
			onCreateStarter={() => {
				void fromStarter
					.mutateAsync({
						path: { organization_id: organizationId },
						body: { starter: 'landscaping' },
					})
					.then((created) => setChosenId(created.data.id))
			}}
		/>
	)
}
