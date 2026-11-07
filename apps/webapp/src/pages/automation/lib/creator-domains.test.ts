import { Wrench } from 'lucide-react'
import { describe, expect, it } from 'vitest'
import type { CreatorKind } from '#/pages/automation/lib/creator'
import {
	creatorDomains,
	domainCopy,
	OTHER_DOMAIN_ID,
} from '#/pages/automation/lib/creator-domains'

function kind(label: string, group: string | null): CreatorKind {
	return {
		id: `connector:${label}`,
		label,
		description: '',
		icon: Wrench,
		hue: 220,
		group,
		source: { type: 'trigger', kind: 'Manual' },
	}
}

describe('creatorDomains', () => {
	it('lists one domain per group, sorted with the French collation, actions sorted by label', () => {
		const domains = creatorDomains([
			kind('Voir un devis', 'Devis'),
			kind('Émettre une facture', 'Factures'),
			kind('Créer un devis', 'Devis'),
			kind('Lire le référentiel', 'Référentiel'),
			kind('Générer un DOE', 'DOE'),
		])

		expect(domains.map((domain) => domain.label)).toEqual([
			'Devis',
			'DOE',
			'Factures',
			'Référentiel',
		])
		expect(domains[0].kinds.map((item) => item.label)).toEqual([
			'Créer un devis',
			'Voir un devis',
		])
	})

	it('describes each domain, and falls back to the name of an unknown group', () => {
		const [known, unknown] = creatorDomains([
			kind('A', 'Achats'),
			kind('B', 'Zèbre'),
		])

		expect(known.description).toBe('Fournisseurs et factures fournisseur')
		expect(unknown).toMatchObject({
			id: 'Zèbre',
			label: 'Zèbre',
			description: '',
		})
		expect(unknown.icon).toBe(Wrench)
		expect(domainCopy('Zèbre').hue).toBe(220)
	})

	it('gathers the actions without a group under Autres', () => {
		const domains = creatorDomains([kind('A', null), kind('B', 'Achats')])

		expect(domains.map((domain) => domain.id)).toEqual([
			'Achats',
			OTHER_DOMAIN_ID,
		])
	})
})
