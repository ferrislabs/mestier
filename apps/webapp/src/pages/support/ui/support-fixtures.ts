import type {
	SupportColumnVM,
	SupportEntryVM,
	SupportMemberVM,
	SupportTicketVM,
} from '#/pages/support/types'

export const ALIX: SupportMemberVM = {
	id: 'member-1',
	name: 'Alix Martin',
	hue: 120,
}

export const JO: SupportMemberVM = {
	id: 'member-2',
	name: 'Jo Durand',
	hue: 40,
}

export function ticketVM(
	overrides: Partial<SupportTicketVM> = {},
): SupportTicketVM {
	return {
		id: 'ticket-1',
		number: 1,
		numberLabel: '#1',
		subject: 'Fuite sous l’évier',
		status: 'new',
		priority: 'normal',
		assignee: null,
		requester: { name: 'Claire Fontaine', email: 'claire@fontaine.fr' },
		badges: [],
		displayFields: [],
		customFields: [],
		preview: 'Bonjour, ma canalisation fuit.',
		externalId: null,
		sourceLabel: 'Saisie manuelle',
		createdAt: '2026-11-01T07:00:00Z',
		updatedAt: '2026-11-01T07:00:00Z',
		...overrides,
	}
}

export function entryVM(
	overrides: Partial<SupportEntryVM> = {},
): SupportEntryVM {
	return {
		id: 'entry-1',
		kind: 'reply',
		authorLabel: 'Alix Martin',
		authorHue: 120,
		createdAt: '2026-11-01T09:00:00Z',
		body: 'Nous passons demain.',
		attachments: [],
		...overrides,
	}
}

export function emptyColumns(): SupportColumnVM[] {
	return [
		{
			status: 'new',
			label: 'Backlog',
			description: 'Reçu, pas encore pris en charge.',
			tickets: [],
		},
		{
			status: 'in_progress',
			label: 'Ouvert',
			description: 'Pris en charge par l’équipe.',
			tickets: [],
		},
		{
			status: 'waiting_customer',
			label: 'En attente client',
			description: 'Nous attendons une réponse du client.',
			tickets: [],
		},
		{
			status: 'resolved',
			label: 'Fermé',
			description: 'Problème résolu.',
			tickets: [],
		},
	]
}
