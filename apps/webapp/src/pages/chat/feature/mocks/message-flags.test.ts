import { renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import {
	resetMessageFlags,
	toggleMessagePin,
	toggleMessageSaved,
	useMessageFlags,
} from '#/pages/chat/feature/mocks/message-flags'

describe('message flags mock', () => {
	beforeEach(() => resetMessageFlags())

	it('pins and unpins a message', async () => {
		const { result } = renderHook(() => useMessageFlags())

		await toggleMessagePin('m-1')
		await waitFor(() =>
			expect(result.current.data.pinned.has('m-1')).toBe(true),
		)

		await toggleMessagePin('m-1')
		await waitFor(() =>
			expect(result.current.data.pinned.has('m-1')).toBe(false),
		)
	})

	it('saves a message without pinning it', async () => {
		const { result } = renderHook(() => useMessageFlags())

		await toggleMessageSaved('m-2')
		await waitFor(() => expect(result.current.data.saved.has('m-2')).toBe(true))

		expect(result.current.data.pinned.has('m-2')).toBe(false)
	})
})
