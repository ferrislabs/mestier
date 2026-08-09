import type { UserManager } from 'oidc-client-ts'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
	adoptRenewalsFromOtherTabs,
	createUserManager,
	getUserManager,
	leadRenewalsForThisTab,
	userStorageKey,
} from '#/lib/oidc'

const AUTHORITY = 'http://idp.test/realms/mestier'
const CLIENT_ID = 'mestier-webapp'

vi.mock('#/lib/runtime-config', () => ({
	getOidcConfiguration: () => ({
		authority: AUTHORITY,
		client_id: CLIENT_ID,
		redirect_uri: 'http://app.test/',
		scope: 'openid profile email',
	}),
}))

interface RenewalStub {
	startSilentRenew: ReturnType<typeof vi.fn>
	getUser: ReturnType<typeof vi.fn>
	events: { unload: ReturnType<typeof vi.fn> }
	settings: { authority: string; client_id: string }
}

function renewalStub(): RenewalStub {
	return {
		startSilentRenew: vi.fn(),
		getUser: vi.fn().mockResolvedValue(null),
		events: { unload: vi.fn().mockResolvedValue(undefined) },
		settings: { authority: AUTHORITY, client_id: CLIENT_ID },
	}
}

function asUserManager(stub: RenewalStub): UserManager {
	return stub as unknown as UserManager
}

function storageEvent(key: string | null, newValue: string | null) {
	return new StorageEvent('storage', { key, newValue })
}

describe('userStorageKey', () => {
	it('matches the key oidc-client-ts actually writes', async () => {
		const manager = createUserManager()
		if (!manager) throw new Error('user manager expected')

		window.localStorage.clear()
		// `storeUser` goes through the real `WebStorageStateStore`, so the key
		// observed here is the library's own, not ours.
		await manager.storeUser({
			toStorageString: () => '{}',
		} as never)

		expect(window.localStorage.key(0)).toBe(
			userStorageKey(AUTHORITY, CLIENT_ID),
		)
	})
})

describe('createUserManager', () => {
	it('never renews on its own — the lock decides who does', () => {
		const manager = createUserManager()

		expect(manager?.settings.automaticSilentRenew).toBe(false)
	})
})

describe('getUserManager', () => {
	it('hands back the same instance, so a remount cannot add a renewer', () => {
		expect(getUserManager()).toBe(getUserManager())
	})
})

describe('leadRenewalsForThisTab', () => {
	beforeEach(() => {
		vi.unstubAllGlobals()
	})

	it('renews only once the lock is held', async () => {
		const stub = renewalStub()
		let release: (() => void) | undefined
		const request = vi.fn(
			(_name: string, callback: () => Promise<never>) =>
				new Promise<void>((resolve) => {
					release = () => {
						void callback()
						resolve()
					}
				}),
		)
		vi.stubGlobal('navigator', { locks: { request } })

		void leadRenewalsForThisTab(asUserManager(stub))
		expect(stub.startSilentRenew).not.toHaveBeenCalled()

		release?.()
		expect(stub.startSilentRenew).toHaveBeenCalledOnce()
	})

	it('holds the lock instead of releasing it to the next tab', async () => {
		const stub = renewalStub()
		let held: Promise<unknown> | undefined
		vi.stubGlobal('navigator', {
			locks: {
				request: (_name: string, callback: () => Promise<never>) => {
					held = callback()
					return new Promise<void>(() => {})
				},
			},
		})

		void leadRenewalsForThisTab(asUserManager(stub))

		const settled = await Promise.race([
			held?.then(() => 'settled'),
			Promise.resolve('pending'),
		])
		expect(settled).toBe('pending')
	})

	it('renews anyway when the browser has no Web Locks', async () => {
		const stub = renewalStub()
		vi.stubGlobal('navigator', {})

		await leadRenewalsForThisTab(asUserManager(stub))

		expect(stub.startSilentRenew).toHaveBeenCalledOnce()
	})
})

describe('adoptRenewalsFromOtherTabs', () => {
	it('adopts the token the leading tab just wrote', () => {
		const stub = renewalStub()
		adoptRenewalsFromOtherTabs(asUserManager(stub))

		window.dispatchEvent(
			storageEvent(
				userStorageKey(AUTHORITY, CLIENT_ID),
				'{"access_token":"…"}',
			),
		)

		expect(stub.getUser).toHaveBeenCalledWith(true)
	})

	it('follows the leading tab out when the session is cleared', () => {
		const stub = renewalStub()
		adoptRenewalsFromOtherTabs(asUserManager(stub))

		window.dispatchEvent(
			storageEvent(userStorageKey(AUTHORITY, CLIENT_ID), null),
		)

		expect(stub.events.unload).toHaveBeenCalledOnce()
		expect(stub.getUser).not.toHaveBeenCalled()
	})

	it('ignores writes that are not the session', () => {
		const stub = renewalStub()
		adoptRenewalsFromOtherTabs(asUserManager(stub))

		window.dispatchEvent(storageEvent('mestier.theme', 'dark'))

		expect(stub.getUser).not.toHaveBeenCalled()
		expect(stub.events.unload).not.toHaveBeenCalled()
	})

	it('stops listening once unsubscribed', () => {
		const stub = renewalStub()
		const stop = adoptRenewalsFromOtherTabs(asUserManager(stub))

		stop()
		window.dispatchEvent(
			storageEvent(
				userStorageKey(AUTHORITY, CLIENT_ID),
				'{"access_token":"…"}',
			),
		)

		expect(stub.getUser).not.toHaveBeenCalled()
	})
})
