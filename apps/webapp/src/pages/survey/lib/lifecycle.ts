export type LifecycleAction = 'open' | 'close' | 'reopen' | 'delete'

export interface LifecycleCopy {
	title: string
	description: string
	confirmLabel: string
	destructive: boolean
	done: string
}

export function lifecycleCopy(
	action: LifecycleAction,
	title: string,
): LifecycleCopy {
	switch (action) {
		case 'open':
			return {
				title: `Ouvrir « ${title} » ?`,
				description:
					'Il acceptera des réponses. Vous pourrez encore modifier ses questions : les réponses déjà reçues gardent la version qu’elles ont vue. Sa visibilité, elle, ne changera plus.',
				confirmLabel: 'Ouvrir aux réponses',
				destructive: false,
				done: 'Questionnaire ouvert aux réponses.',
			}
		case 'close':
			return {
				title: `Fermer « ${title} » ?`,
				description:
					'Il n’acceptera plus aucune réponse. Vous pourrez le rouvrir plus tard.',
				confirmLabel: 'Fermer le questionnaire',
				destructive: true,
				done: 'Questionnaire fermé.',
			}
		case 'reopen':
			return {
				title: 'Rouvrir ce questionnaire ?',
				description: 'Les répondants pourront de nouveau répondre.',
				confirmLabel: 'Rouvrir',
				destructive: false,
				done: 'Questionnaire rouvert.',
			}
		case 'delete':
			return {
				title: `Supprimer « ${title} » ?`,
				description:
					'La suppression est définitive. Un questionnaire qui a déjà reçu des réponses ne se supprime pas : fermez-le.',
				confirmLabel: 'Supprimer',
				destructive: true,
				done: 'Questionnaire supprimé.',
			}
	}
}
