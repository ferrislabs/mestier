// MOCK: external services — no backend endpoint yet. Replace with the OpenAPI-backed service catalogue once Mestier supports provider specs.
export interface ExternalService {
	id: string
	label: string
	description: string
	hue: number
}

export const MOCK_EXTERNAL_SERVICES: readonly ExternalService[] = [
	{
		id: 'google',
		label: 'Google',
		description: 'Gmail, Agenda, Drive',
		hue: 220,
	},
	{
		id: 'microsoft',
		label: 'Microsoft Outlook',
		description: 'Messagerie, calendrier, OneDrive',
		hue: 210,
	},
	{
		id: 'discord',
		label: 'Discord',
		description: 'Messages, salons',
		hue: 260,
	},
	{ id: 'slack', label: 'Slack', description: 'Messages, canaux', hue: 320 },
	{ id: 'stripe', label: 'Stripe', description: 'Paiements', hue: 270 },
	{
		id: 'qonto',
		label: 'Qonto',
		description: 'Banque, transactions',
		hue: 240,
	},
]
