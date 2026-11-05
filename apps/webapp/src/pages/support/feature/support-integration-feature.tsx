import { useState } from 'react'
import {
	createApiKey,
	revokeApiKey,
	sendTestRequest,
	useApiKeys,
} from '#/pages/support/feature/mocks/api-keys'
import {
	generateSecret,
	validateApiPayload,
} from '#/pages/support/lib/api-example'
import type { SupportIntegrationTestResult } from '#/pages/support/types'
import { SupportIntegrationUI } from '#/pages/support/ui/support-integration-ui'

export function SupportIntegrationFeature() {
	const keys = useApiKeys()
	const [revealed, setRevealed] = useState<{
		name: string
		secret: string
	} | null>(null)
	const [testResult, setTestResult] =
		useState<SupportIntegrationTestResult | null>(null)
	const [isSending, setIsSending] = useState(false)

	async function handleCreateKey(name: string) {
		const secret = generateSecret()
		const key = await createApiKey(name, secret)
		setRevealed({ name: key.name, secret })
	}

	async function handleSendTest(keyId: string, payload: string) {
		let raw: unknown
		try {
			raw = JSON.parse(payload)
		} catch {
			setTestResult({ kind: 'invalid', errors: ['JSON invalide.'] })
			return
		}

		const validation = validateApiPayload(raw)
		if (!validation.ok) {
			setTestResult({ kind: 'invalid', errors: validation.errors })
			return
		}

		setIsSending(true)
		const outcome = await sendTestRequest(keyId, validation.payload.externalId)
		setIsSending(false)
		setTestResult({ kind: outcome.kind, ticketNumber: outcome.ticketNumber })
	}

	return (
		<SupportIntegrationUI
			origin={window.location.origin}
			keys={[...keys.data]}
			revealed={revealed}
			testResult={testResult}
			isSending={isSending}
			onCreateKey={(name) => void handleCreateKey(name)}
			onDismissRevealed={() => setRevealed(null)}
			onRevokeKey={(keyId) => void revokeApiKey(keyId)}
			onSendTest={(keyId, payload) => void handleSendTest(keyId, payload)}
		/>
	)
}
