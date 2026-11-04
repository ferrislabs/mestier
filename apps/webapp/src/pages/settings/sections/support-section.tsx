import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { useSupportSettings } from '#/hooks/use-support'
import { useUpdateSupportSettings } from '#/hooks/use-support-settings'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	parseSupportDisplayFields,
	supportDisplayFieldsToText,
} from '#/pages/settings/lib/support'
import { SupportSettingsUI } from '#/pages/settings/ui/support-settings-ui'

export function SupportSection() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<div className="flex flex-col gap-8" key={activeOrganization.id}>
			<SettingsPanel organizationId={activeOrganization.id} />
		</div>
	)
}

function SettingsPanel({ organizationId }: { organizationId: string }) {
	const settingsQuery = useSupportSettings(organizationId)
	const updateSettings = useUpdateSupportSettings(organizationId)
	const canManage = useHasPermission('MANAGE_SUPPORT')
	const [draft, setDraft] = useState<string | null>(null)
	const [formError, setFormError] = useState<string | null>(null)

	const displayFields = settingsQuery.data?.data.display_fields
	const isLoaded = displayFields !== undefined
	const value =
		draft ?? (isLoaded ? supportDisplayFieldsToText(displayFields) : '')

	return (
		<SupportSettingsUI
			isLoading={settingsQuery.isLoading}
			value={value}
			canManage={canManage && isLoaded}
			isPending={updateSettings.isPending}
			formError={formError}
			saveError={
				mutationErrorMessage(updateSettings.error) ??
				(isLoaded ? null : mutationErrorMessage(settingsQuery.error))
			}
			onChange={setDraft}
			onSubmit={() => {
				if (!canManage || !isLoaded) return
				const result = parseSupportDisplayFields(value)
				if (!result.ok) {
					setFormError(result.error)
					return
				}
				setFormError(null)
				updateSettings.mutate({
					path: { organization_id: organizationId },
					body: { display_fields: result.fields },
				})
			}}
		/>
	)
}
