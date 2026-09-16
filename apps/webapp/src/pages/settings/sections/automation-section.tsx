import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useAutomationSettings,
	useUpdateAutomationSettings,
} from '#/hooks/use-automation'
import {
	parseSettingsForm,
	settingsToFormValues,
} from '#/pages/settings/lib/automation'
import { AutomationSettingsUI } from '#/pages/settings/ui/automation-settings-ui'

export function AutomationSection() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<div className="flex flex-col gap-8" key={activeOrganization.id}>
			<SettingsPanel organizationId={activeOrganization.id} />
		</div>
	)
}

function SettingsPanel({ organizationId }: { organizationId: string }) {
	const settingsQuery = useAutomationSettings(organizationId)
	const updateSettings = useUpdateAutomationSettings()
	const [draft, setDraft] = useState<ReturnType<
		typeof settingsToFormValues
	> | null>(null)
	const [formError, setFormError] = useState<string | null>(null)

	const settings = settingsQuery.data?.data
	const values = draft ?? (settings ? settingsToFormValues(settings) : null)

	if (!values) {
		return (
			<AutomationSettingsUI
				isLoading={settingsQuery.isLoading}
				values={{
					eventRetentionSeconds: '',
					succeededRunRetentionSeconds: '',
					retryScheduleSeconds: '',
					disableTargetAfter: '',
				}}
				retrySchedulePreview={[]}
				isPending={false}
				formError={null}
				saveError={settingsQuery.error?.message ?? null}
				onChange={() => {}}
				onSubmit={() => {}}
			/>
		)
	}

	const parsed = parseSettingsForm(values)

	return (
		<AutomationSettingsUI
			isLoading={false}
			values={values}
			retrySchedulePreview={parsed.ok ? parsed.body.retry_schedule_seconds : []}
			isPending={updateSettings.isPending}
			formError={formError}
			saveError={updateSettings.error?.message ?? null}
			onChange={(patch) => setDraft({ ...values, ...patch })}
			onSubmit={() => {
				const result = parseSettingsForm(values)
				if (!result.ok) {
					setFormError(result.error)
					return
				}
				setFormError(null)
				void updateSettings.mutateAsync({
					path: { organization_id: organizationId },
					body: result.body,
				})
			}}
		/>
	)
}
