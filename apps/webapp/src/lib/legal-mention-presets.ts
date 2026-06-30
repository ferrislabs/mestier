export interface LegalMentionPreset {
	id: string
	label: string
	name: string
	body: string
}

export const LEGAL_MENTION_PRESETS: LegalMentionPreset[] = [
	{
		id: 'late-payment-penalties',
		label: 'Pénalités de retard (L.441-10)',
		name: 'Pénalités de retard',
		body: "En cas de retard de paiement, des pénalités de retard sont exigibles dès le premier jour suivant la date d'échéance, au taux de trois fois le taux d'intérêt légal en vigueur (art. L.441-10 du Code de commerce). Une indemnité forfaitaire pour frais de recouvrement de 40 € sera également due de plein droit (art. D.441-5 du Code de commerce).",
	},
	{
		id: 'vat-exemption-293b',
		label: 'TVA non applicable (art. 293 B du CGI)',
		name: 'TVA non applicable',
		body: 'TVA non applicable — article 293 B du Code général des impôts. Le présent document ne comporte pas de TVA, le prestataire bénéficiant du régime de la franchise en base de TVA.',
	},
	{
		id: 'deposit-clause',
		label: "Clause d'acompte",
		name: 'Acompte',
		body: "Un acompte est requis à la signature du présent devis. Le montant de l'acompte sera précisé sur le document contractuel. L'acompte versé est acquis en cas d'annulation à l'initiative du client plus de 48 heures avant le début de la prestation.",
	},
	{
		id: 'retention-of-title',
		label: 'Réserve de propriété',
		name: 'Clause de réserve de propriété',
		body: "Les marchandises livrées demeurent la propriété du vendeur jusqu'au paiement intégral du prix en principal et accessoires, conformément à la loi n° 80-335 du 12 mai 1980. Le défaut de paiement pourra entraîner la revendication de ces biens. Le transfert de risques intervient dès la livraison.",
	},
	{
		id: 'consumer-mediation',
		label: 'Médiation de la consommation (L.612-1)',
		name: 'Médiation de la consommation',
		body: "Conformément à l'article L.612-1 du Code de la consommation, tout consommateur a le droit de recourir gratuitement à un médiateur de la consommation en vue de la résolution amiable du litige qui l'oppose à un professionnel. En cas de litige non résolu, le client particulier peut saisir le médiateur compétent dont les coordonnées seront communiquées sur demande.",
	},
]
