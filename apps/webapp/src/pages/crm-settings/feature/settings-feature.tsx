import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCrmSettings,
	useRaiseSequences,
	useReplaceStripeSecret,
	useUpdateBilling,
	useUpdateCompany,
} from '#/pages/crm-settings/feature/mocks/settings'
import { sequenceRows } from '#/pages/crm-settings/lib/settings-model'
import { SettingsUI } from '#/pages/crm-settings/ui/settings-ui'
import { todayIso } from '#/pages/customers/lib/format'

export function SettingsFeature() {
	const { activeOrganization } = useActiveOrganization()
	const state = useCrmSettings().data
	const updateCompany = useUpdateCompany()
	const updateBilling = useUpdateBilling()
	const replaceSecret = useReplaceStripeSecret()
	const raiseSequences = useRaiseSequences()
	const year = Number(todayIso().slice(0, 4))

	return (
		<SettingsUI
			organizationSlug={activeOrganization.slug}
			settings={state.settings}
			year={year}
			sequenceRows={sequenceRows(state, year)}
			onSaveCompany={updateCompany.mutateAsync}
			onSaveBilling={updateBilling.mutateAsync}
			onRaiseSequences={(raises) => raiseSequences.mutateAsync(year, raises)}
			onReplaceSecret={replaceSecret.mutateAsync}
		/>
	)
}
