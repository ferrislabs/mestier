import {
	Boxes,
	Database,
	FileBadge,
	FileKey,
	Fingerprint,
	FolderUp,
	Inbox,
	KeyRound,
	Link2,
	LockKeyhole,
	Send,
	ServerCog,
	Settings2,
	Shield,
	ShieldCheck,
	Signature,
	Terminal,
	Ticket,
} from 'lucide-react'
import { describe, expect, it } from 'vitest'
import { credentialAppearance } from '#/pages/automation/lib/credential-appearance'

describe('credentialAppearance', () => {
	it('gives each kind of the server the icon and hue the prototype gives its equivalent', () => {
		const expected = {
			http_basic: [LockKeyhole, 250],
			bearer_token: [Ticket, 280],
			http_header: [KeyRound, 200],
			api_key_query: [Link2, 170],
			digest: [Fingerprint, 230],
			oauth2_authorization_code: [ShieldCheck, 150],
			oauth2_client_credentials: [ServerCog, 130],
			oauth1: [Shield, 110],
			jwt: [FileKey, 310],
			mtls: [FileBadge, 50],
			custom_auth: [Settings2, 20],
			hmac: [Signature, 335],
			smtp: [Send, 25],
			imap: [Inbox, 40],
			ssh: [Terminal, 0],
			ftp: [FolderUp, 60],
			sql: [Database, 210],
		} as const

		for (const [code, [icon, hue]] of Object.entries(expected)) {
			expect(credentialAppearance(code), code).toEqual({ icon, hue })
		}
	})

	it('matches the Odoo connectors look for the Odoo key', () => {
		expect(credentialAppearance('odoo_api')).toEqual({ icon: Boxes, hue: 300 })
	})

	it('does not guess a look from words in the code', () => {
		expect(credentialAppearance('digest_auth')).toEqual(
			credentialAppearance('mystery'),
		)
	})

	it('falls back for an unknown kind', () => {
		expect(credentialAppearance('mystery')).toEqual({
			icon: KeyRound,
			hue: 200,
		})
	})
})
