import {
	createMockStore,
	resolveAfterDelay,
} from '#/pages/customers/feature/mocks/mock-store'

const store = createMockStore<Readonly<Record<string, string>>>({})

export function useStripeCustomerIds(): Readonly<Record<string, string>> {
	return store.use()
}

export function useStripeCustomerId(customerId: string): string | null {
	return store.use()[customerId] ?? null
}

export function useSaveStripeCustomerId() {
	return {
		isPending: false,
		mutateAsync: async (
			customerId: string,
			stripeCustomerId: string | null,
		) => {
			await resolveAfterDelay(null)
			store.set((current) => {
				const { [customerId]: _removed, ...rest } = current
				return stripeCustomerId === null
					? rest
					: { ...rest, [customerId]: stripeCustomerId }
			})
		},
	}
}
