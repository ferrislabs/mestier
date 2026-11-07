import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useBillingSettings } from '#/hooks/use-billing-settings'
import {
	billingOf,
	DEFAULT_BILLING,
	type StripeValues,
	settingsOf,
} from '#/pages/crm-settings/lib/settings-api'
import type { Settings } from '#/pages/crm-settings/types'
import {
	createMockStore,
	resolveAfterDelay,
} from '#/pages/customers/feature/mocks/mock-store'

const stripeStore = createMockStore({ secretSet: true })

export function useStripeSettings(organizationSlug: string): StripeValues {
	const { secretSet } = stripeStore.use()
	return {
		stripeEndpoint: `https://api.mestier.app/webhooks/stripe/${organizationSlug}`,
		stripeSecretSet: secretSet,
	}
}

export function useCrmSettings(): {
	data: { settings: Settings }
	isLoading: boolean
} {
	const { activeOrganization } = useActiveOrganization()
	const billing = useBillingSettings(activeOrganization.id)
	const stripe = useStripeSettings(activeOrganization.slug)
	const values = billing.data ? billingOf(billing.data.data) : DEFAULT_BILLING
	return {
		data: { settings: settingsOf(activeOrganization, values, stripe) },
		isLoading: billing.isLoading,
	}
}

export function useReplaceStripeSecret() {
	return {
		isPending: false,
		mutateAsync: async () => {
			await resolveAfterDelay(null)
			stripeStore.set({ secretSet: true })
		},
	}
}
