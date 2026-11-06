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
	type LucideIcon,
	Send,
	ServerCog,
	Settings2,
	Shield,
	ShieldCheck,
	Signature,
	Terminal,
	Ticket,
} from 'lucide-react'

export interface CredentialAppearance {
	icon: LucideIcon
	hue: number
}

const APPEARANCES: Readonly<Record<string, CredentialAppearance>> = {
	http_basic: { icon: LockKeyhole, hue: 250 },
	bearer_token: { icon: Ticket, hue: 280 },
	http_header: { icon: KeyRound, hue: 200 },
	api_key_query: { icon: Link2, hue: 170 },
	digest: { icon: Fingerprint, hue: 230 },
	oauth2_authorization_code: { icon: ShieldCheck, hue: 150 },
	oauth2_client_credentials: { icon: ServerCog, hue: 130 },
	oauth1: { icon: Shield, hue: 110 },
	jwt: { icon: FileKey, hue: 310 },
	mtls: { icon: FileBadge, hue: 50 },
	custom_auth: { icon: Settings2, hue: 20 },
	hmac: { icon: Signature, hue: 335 },
	smtp: { icon: Send, hue: 25 },
	imap: { icon: Inbox, hue: 40 },
	ssh: { icon: Terminal, hue: 0 },
	ftp: { icon: FolderUp, hue: 60 },
	sql: { icon: Database, hue: 210 },
	odoo_api: { icon: Boxes, hue: 300 },
}

const FALLBACK: CredentialAppearance = { icon: KeyRound, hue: 200 }

export function credentialAppearance(kind: string): CredentialAppearance {
	return APPEARANCES[kind] ?? FALLBACK
}
