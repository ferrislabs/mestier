import {
	type ExternalService,
	MOCK_EXTERNAL_SERVICES,
} from '#/pages/automation/feature/mocks/external-services'

export function useExternalServices(): {
	data: readonly ExternalService[]
	isLoading: boolean
} {
	return { data: MOCK_EXTERNAL_SERVICES, isLoading: false }
}
