import type { StripeValues } from '#/pages/crm-settings/lib/settings-api'
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

export function useReplaceStripeSecret() {
	return {
		isPending: false,
		mutateAsync: async () => {
			await resolveAfterDelay(null)
			stripeStore.set({ secretSet: true })
		},
	}
}
