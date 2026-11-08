// MOCK: home activity feed — no backend endpoint yet. Replace with an endpoint aggregating what each module wants followed (late invoices, pending leave requests, unread channels…).
import type { ModuleId } from '#/modules/types'
import {
	identificationsPath,
	workflowsPath,
} from '#/pages/automation/lib/automation-paths'
import type { ActivityState } from '#/pages/home/types'

export interface MockActivity {
	id: string
	moduleId: ModuleId
	title: string
	context?: string
	state: ActivityState
	when?: string
	time?: string
	progress?: number
	to: string
}

export const MOCK_HOME_ACTIVITY: MockActivity[] = [
	{
		id: 'auto-relance',
		moduleId: 'automatisation',
		title: 'Workflow « Relance des devis » en échec',
		context:
			'Étape « Envoyer l’e-mail de relance » : identification SMTP expirée',
		state: 'failed',
		when: 'Il y a 2 heures',
		to: workflowsPath(),
	},
	{
		id: 'auto-smtp',
		moduleId: 'automatisation',
		title: 'Identification « SMTP atelier » expirée',
		context: 'Utilisée par 3 workflows',
		state: 'late',
		to: identificationsPath(),
	},
	{
		id: 'crm-f112',
		moduleId: 'crm',
		title: 'Facture F-2026-112 impayée',
		context: 'SCI Les Tilleuls, 3 991,90 €',
		state: 'late',
		when: 'Échue depuis le 23 octobre',
		to: '/crm/factures',
	},
	{
		id: 'crm-stripe-review',
		moduleId: 'crm',
		title: 'Paiement Stripe à rapprocher',
		context: 'Atelier Bois & Co, 490 €',
		state: 'waiting',
		when: 'Reçu le 3 novembre',
		to: '/crm/factures',
	},
	{
		id: 'crm-d210',
		moduleId: 'crm',
		title: 'Devis D-2026-210 en attente de signature',
		context: 'Boulangerie Lemoine, 9 057,60 €',
		state: 'waiting',
		when: 'Envoyé lundi',
		to: '/crm/devis',
	},
	{
		id: 'crm-d209',
		moduleId: 'crm',
		title: 'Relancer le devis D-2026-209',
		context: 'M. et Mme Fabre, 2 187,90 €',
		state: 'today',
		time: '14:00',
		to: '/crm/devis',
	},
	{
		id: 'chat-dupont',
		moduleId: 'chat',
		title: '5 messages non lus dans #chantier-dupont',
		context: 'Dernier message de Karim Benali',
		state: 'today',
		when: 'Il y a 10 minutes',
		to: '/chat',
	},
	{
		id: 'planif-dupont',
		moduleId: 'projet',
		title: 'Rénovation cuisine Dupont',
		context: 'Étape en cours : pose des meubles',
		state: 'progress',
		progress: 60,
		when: 'Fin prévue le 14 novembre',
		to: '/projet',
	},
	{
		id: 'planif-ferry',
		moduleId: 'projet',
		title: 'Mise aux normes électriques, école Jules Ferry',
		context: 'Étape en cours : diagnostic',
		state: 'progress',
		progress: 25,
		when: 'Fin prévue le 20 décembre',
		to: '/projet',
	},
	{
		id: 'planif-entretien',
		moduleId: 'planification',
		title: '3 visites d’entretien à programmer',
		context: 'Contrats chaudière, échéance novembre',
		state: 'late',
		when: 'Depuis le 1er novembre',
		to: '/planification/maintenance',
	},
	{
		id: 'planning-lilas',
		moduleId: 'planning',
		title: 'Pose de fenêtres, 12 rue des Lilas',
		context: 'Julien Roche, Karim Benali',
		state: 'today',
		time: '08:30',
		to: '/planning/calendar',
	},
	{
		id: 'planning-moreau',
		moduleId: 'planning',
		title: 'Dépannage chauffe-eau, famille Moreau',
		context: 'Lucie Martin',
		state: 'today',
		time: '11:00',
		to: '/planning/calendar',
	},
	{
		id: 'planning-ecart',
		moduleId: 'planning',
		title: 'Écart signalé sur le chantier Dupont',
		context: '2 h de plus que prévu mardi',
		state: 'late',
		when: 'Signalé hier',
		to: '/planning/reports',
	},
	{
		id: 'hr-julien-conges',
		moduleId: 'hr',
		title: 'Demande de congés de Julien Roche',
		context: '2 jours de congés payés, du 5 au 6 novembre',
		state: 'waiting',
		when: 'À valider',
		to: '/rh/absences',
	},
	{
		id: 'hr-sophie-essai',
		moduleId: 'hr',
		title: 'Fin de période d’essai de Sophie Lambert',
		context: 'Le 6 novembre : confirmer ou rompre avant cette date',
		state: 'today',
		when: 'Cette semaine',
		to: '/rh/membres',
	},
	{
		id: 'support-58',
		moduleId: 'support',
		title: 'Ticket #58 : volet roulant bloqué',
		context: 'Le client a répondu, brouillon en attente d’avis',
		state: 'today',
		when: 'Ce matin',
		to: '/support',
	},
	{
		id: 'support-unassigned',
		moduleId: 'support',
		title: '3 tickets sans responsable',
		context: 'Fuite chauffe-eau, devis pergola, téléchargement du devis',
		state: 'waiting',
		when: 'Depuis hier',
		to: '/support',
	},
]
