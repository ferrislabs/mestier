import { toast } from 'sonner'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useFeatures, useSetSelfServiceFeature } from '#/hooks/use-features'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import { FeaturesSettingsUI } from '#/pages/settings/ui/features-settings-ui'

export function FeaturesSection() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<div className="flex flex-col gap-6" key={activeOrganization.id}>
			<FeaturesPanel organizationId={activeOrganization.id} />
		</div>
	)
}

function FeaturesPanel({ organizationId }: { organizationId: string }) {
	const { selfService, isLoading, error } = useFeatures()
	const setFeature = useSetSelfServiceFeature(organizationId)
	const canManage = useHasPermission('MANAGE_ORG')

	return (
		<FeaturesSettingsUI
			isLoading={isLoading}
			features={selfService}
			canManage={canManage}
			pendingKey={
				setFeature.isPending ? (setFeature.variables?.path.flag ?? null) : null
			}
			loadError={mutationErrorMessage(error)}
			onToggle={(flag, enabled) => {
				if (!canManage) return
				setFeature.mutate(
					{
						path: { organization_id: organizationId, flag },
						body: { enabled },
					},
					{
						onSuccess: () =>
							toast.success(
								enabled
									? 'Fonctionnalité activée.'
									: 'Fonctionnalité désactivée.',
							),
						onError: (cause) =>
							toast.error(
								mutationErrorMessage(cause) ?? 'Une erreur est survenue.',
								{
									duration: 6000,
								},
							),
					},
				)
			}}
		/>
	)
}
