// MOCK: rh — no backend endpoint yet. Replace with the HR domain API (contracts, leave accounts, absence requests, overtime declarations, day planning).
import { NO_SCHEDULE, weekly } from '#/pages/hr/lib/rh/contract-schedule'
import { daysInRange, weekdayOf } from '#/pages/hr/lib/rh/dates'
import { isHoliday } from '#/pages/hr/lib/rh/holidays'
import { EMPTY_ADDRESS } from '#/pages/hr/lib/rh/members'
import { DEFAULT_LUNCH } from '#/pages/hr/lib/rh/schedule'
import type {
	Absence,
	AbsenceType,
	BalanceAdjustment,
	Contract,
	DayOverride,
	ExtraPeriod,
	LeaveAccount,
	Member,
	RhState,
	Surcharge,
	WeekSchedule,
	WorkDay,
} from '#/pages/hr/lib/rh/types'

export const CURRENT_MEMBER_ID = 'baptiste'

const FULL_DAY: WorkDay = { start: '08:00', end: '17:00', lunch: DEFAULT_LUNCH }
const SHORT_DAY: WorkDay = {
	start: '08:00',
	end: '16:00',
	lunch: DEFAULT_LUNCH,
}
const OFFICE_DAY: WorkDay = {
	start: '09:00',
	end: '17:30',
	lunch: { start: '12:30', end: '13:30' },
}

const FULL_WEEK: WeekSchedule = [
	FULL_DAY,
	FULL_DAY,
	FULL_DAY,
	FULL_DAY,
	FULL_DAY,
	null,
	null,
]

const SHORT_WEEK: WeekSchedule = [
	FULL_DAY,
	FULL_DAY,
	FULL_DAY,
	FULL_DAY,
	null,
	null,
	null,
]

const FOUR_DAYS: WeekSchedule = [
	OFFICE_DAY,
	OFFICE_DAY,
	OFFICE_DAY,
	OFFICE_DAY,
	null,
	null,
	null,
]

const APPRENTICE_WEEK: WeekSchedule = [
	SHORT_DAY,
	SHORT_DAY,
	SHORT_DAY,
	SHORT_DAY,
	SHORT_DAY,
	null,
	null,
]

function contract(
	id: string,
	fields: Pick<Contract, 'type' | 'startDate'> & Partial<Contract>,
): Contract {
	return {
		endDate: null,
		trialEndDate: null,
		schedule: weekly(FULL_WEEK),
		...fields,
		id,
	}
}

function member(
	fields: Pick<
		Member,
		| 'id'
		| 'firstName'
		| 'lastName'
		| 'jobTitle'
		| 'hue'
		| 'contracts'
		| 'managerId'
	> &
		Partial<Member>,
): Member {
	const email = `${fields.firstName}.${fields.lastName}`
		.toLowerCase()
		.normalize('NFD')
		.replace(/[\u0300-\u036f]/g, '')
	return {
		workEmail: `${email}@atelier-roche.fr`,
		personalEmail: '',
		mobile: '06 12 34 56 78',
		landline: '',
		address: { ...EMPTY_ADDRESS },
		workPermitEndDate: null,
		status: 'active',
		archivedAt: null,
		documents: [],
		...fields,
	}
}

const MEMBERS: readonly Member[] = [
	member({
		id: 'baptiste',
		firstName: 'Baptiste',
		lastName: 'Parmantier',
		jobTitle: 'Gérant',
		hue: 250,
		contracts: [
			contract('c-baptiste-1', { type: 'gerant', startDate: '2014-03-01' }),
		],
		managerId: null,
		mobile: '06 41 22 18 90',
		personalEmail: 'baptiste.parmantier@gmail.com',
		landline: '04 70 31 24 18',
		address: {
			street: '14 avenue de la Gare',
			complement: '',
			postalCode: '03200',
			city: 'Vichy',
			country: 'France',
		},
		documents: [
			{
				id: 'doc-baptiste-1',
				name: 'Statuts de la société.pdf',
				kind: 'other',
				addedAt: '2014-03-01',
			},
		],
	}),
	member({
		id: 'karim',
		firstName: 'Karim',
		lastName: 'Benali',
		jobTitle: 'Chef de chantier',
		hue: 35,
		contracts: [
			contract('c-karim-1', { type: 'cdi', startDate: '2017-05-15' }),
		],
		managerId: 'baptiste',
		mobile: '06 72 15 40 03',
		personalEmail: 'karim.benali03@gmail.com',
		landline: '',
		address: {
			street: '8 rue des Acacias',
			complement: 'Résidence Les Tilleuls',
			postalCode: '03300',
			city: 'Cusset',
			country: 'France',
		},
		documents: [
			{
				id: 'doc-karim-1',
				name: 'Contrat de travail CDI.pdf',
				kind: 'contract',
				addedAt: '2017-05-15',
			},
			{
				id: 'doc-karim-2',
				name: 'Visite médicale 2025.pdf',
				kind: 'medical',
				addedAt: '2025-02-11',
			},
		],
	}),
	member({
		id: 'julien',
		firstName: 'Julien',
		lastName: 'Roche',
		jobTitle: 'SAV et pose',
		hue: 155,
		contracts: [
			contract('c-julien-1', {
				type: 'apprentissage',
				startDate: '2019-09-02',
				endDate: '2021-08-31',
				schedule: weekly(APPRENTICE_WEEK),
			}),
			contract('c-julien-2', {
				type: 'cdd',
				startDate: '2021-09-01',
				endDate: '2022-02-28',
			}),
			contract('c-julien-3', { type: 'cdi', startDate: '2022-03-01' }),
		],
		managerId: 'baptiste',
		mobile: '06 33 87 12 45',
		personalEmail: 'julien.roche@outlook.fr',
		landline: '',
		address: {
			street: '27 boulevard des États-Unis',
			complement: '',
			postalCode: '03200',
			city: 'Vichy',
			country: 'France',
		},
		documents: [
			{
				id: 'doc-julien-1',
				name: 'Contrat d’apprentissage.pdf',
				kind: 'contract',
				addedAt: '2019-09-02',
			},
			{
				id: 'doc-julien-2',
				name: 'Contrat de travail CDI.pdf',
				kind: 'contract',
				addedAt: '2022-03-01',
			},
		],
	}),
	member({
		id: 'lucie',
		firstName: 'Lucie',
		lastName: 'Martin',
		jobTitle: 'Administration',
		hue: 350,
		contracts: [
			contract('c-lucie-1', {
				type: 'cdi',
				startDate: '2021-01-04',
				endDate: '2023-08-31',
			}),
			contract('c-lucie-2', {
				type: 'cdi',
				startDate: '2023-09-01',
				schedule: weekly(FOUR_DAYS),
			}),
		],
		managerId: 'baptiste',
		mobile: '06 58 03 77 21',
		personalEmail: 'lucie.martin.perso@gmail.com',
		landline: '04 70 98 12 33',
		address: {
			street: '3 impasse du Moulin',
			complement: '',
			postalCode: '03700',
			city: 'Bellerive-sur-Allier',
			country: 'France',
		},
		documents: [
			{
				id: 'doc-lucie-1',
				name: 'Avenant temps partiel 80 %.pdf',
				kind: 'contract',
				addedAt: '2023-09-01',
			},
		],
	}),
	member({
		id: 'sophie',
		firstName: 'Sophie',
		lastName: 'Lambert',
		jobTitle: 'Plombière chauffagiste',
		hue: 200,
		contracts: [
			contract('c-sophie-1', {
				type: 'cdi',
				startDate: '2026-07-06',
				trialEndDate: '2026-11-06',
			}),
		],
		managerId: 'karim',
		mobile: '06 90 11 25 64',
		personalEmail: 'sophie.lambert@proton.me',
		landline: '',
		address: {
			street: '41 rue Lucas',
			complement: 'Bâtiment B',
			postalCode: '03200',
			city: 'Vichy',
			country: 'France',
		},
	}),
	member({
		id: 'thomas',
		firstName: 'Thomas',
		lastName: 'Girard',
		jobTitle: 'Électricien',
		hue: 70,
		contracts: [
			contract('c-thomas-1', {
				type: 'apprentissage',
				startDate: '2021-09-01',
				endDate: '2023-05-31',
				schedule: weekly(APPRENTICE_WEEK),
			}),
			contract('c-thomas-2', { type: 'cdi', startDate: '2023-06-01' }),
		],
		managerId: 'karim',
		mobile: '06 27 64 90 38',
		personalEmail: 'thomas.girard@gmail.com',
		landline: '',
		address: {
			street: '19 rue de Belgique',
			complement: '',
			postalCode: '03200',
			city: 'Vichy',
			country: 'France',
		},
	}),
	member({
		id: 'hugo',
		firstName: 'Hugo',
		lastName: 'Faure',
		jobTitle: 'Menuisier',
		hue: 300,
		contracts: [
			contract('c-hugo-1', {
				type: 'cdd',
				startDate: '2026-05-04',
				endDate: '2026-12-18',
				schedule: {
					kind: 'rotation',
					anchor: '2026-05-04',
					blocks: [
						{ length: 1, unit: 'weeks', label: 'Semaine A', week: FULL_WEEK },
						{ length: 1, unit: 'weeks', label: 'Semaine B', week: SHORT_WEEK },
					],
				},
			}),
		],
		workPermitEndDate: '2026-12-15',
		managerId: 'karim',
		mobile: '06 19 48 02 76',
		personalEmail: '',
		landline: '',
		address: {
			street: '5 chemin des Vignes',
			complement: '',
			postalCode: '03270',
			city: 'Saint-Yorre',
			country: 'France',
		},
	}),
	member({
		id: 'yanis',
		firstName: 'Yanis',
		lastName: 'Moreau',
		jobTitle: 'Apprenti plombier',
		hue: 20,
		contracts: [
			contract('c-yanis-1', {
				type: 'apprentissage',
				startDate: '2025-09-01',
				endDate: '2027-08-31',
				schedule: {
					kind: 'rotation',
					anchor: '2025-09-01',
					blocks: [
						{ length: 1, unit: 'weeks', label: 'CFA', week: NO_SCHEDULE },
						{
							length: 3,
							unit: 'weeks',
							label: 'Entreprise',
							week: APPRENTICE_WEEK,
						},
					],
				},
			}),
		],
		managerId: 'karim',
		mobile: '07 83 52 16 09',
		personalEmail: 'yanis.moreau@gmail.com',
		landline: '04 70 59 41 02',
		address: {
			street: '22 rue du Maréchal Foch',
			complement: '',
			postalCode: '03300',
			city: 'Cusset',
			country: 'France',
		},
	}),
	member({
		id: 'manon',
		firstName: 'Manon',
		lastName: 'Petit',
		jobTitle: 'Aide poseuse',
		hue: 100,
		contracts: [
			contract('c-manon-1', {
				type: 'interim',
				startDate: '2026-10-12',
				endDate: '2026-11-27',
				schedule: { kind: 'variable', defaultDay: FULL_DAY },
			}),
		],
		managerId: 'julien',
		mobile: '07 12 90 33 58',
		personalEmail: 'manon.petit@outlook.fr',
		landline: '',
		address: {
			street: '10 rue de Nîmes',
			complement: '',
			postalCode: '03200',
			city: 'Vichy',
			country: 'France',
		},
	}),
	member({
		id: 'marc',
		firstName: 'Marc',
		lastName: 'Tessier',
		jobTitle: 'Peintre',
		hue: 230,
		contracts: [
			contract('c-marc-1', {
				type: 'cdi',
				startDate: '2018-02-05',
				endDate: '2026-06-30',
			}),
		],
		managerId: 'karim',
		status: 'archived',
		archivedAt: '2026-06-30',
		mobile: '06 55 31 87 12',
		personalEmail: 'marc.tessier@gmail.com',
		landline: '',
		address: {
			street: '7 route de Thiers',
			complement: '',
			postalCode: '03300',
			city: 'Creuzier-le-Vieux',
			country: 'France',
		},
	}),
]

const ABSENCE_TYPES: readonly AbsenceType[] = [
	{
		id: 'cp',
		name: 'Congés payés',
		hue: 155,
		countMode: 'working',
		accountId: 'cp',
		paid: true,
		needsApproval: true,
		needsProof: false,
		archived: false,
	},
	{
		id: 'rtt',
		name: 'RTT',
		hue: 250,
		countMode: 'working',
		accountId: 'rtt',
		paid: true,
		needsApproval: true,
		needsProof: false,
		archived: false,
	},
	{
		id: 'maladie',
		name: 'Arrêt maladie',
		hue: 25,
		countMode: 'calendar',
		accountId: null,
		paid: true,
		needsApproval: false,
		needsProof: true,
		archived: false,
	},
	{
		id: 'intemperies',
		name: 'Intempéries',
		hue: 215,
		countMode: 'working',
		accountId: null,
		paid: true,
		needsApproval: false,
		needsProof: false,
		archived: false,
	},
	{
		id: 'evenement',
		name: 'Événement familial',
		hue: 320,
		countMode: 'working',
		accountId: null,
		paid: true,
		needsApproval: true,
		needsProof: true,
		archived: false,
	},
	{
		id: 'formation',
		name: 'Formation',
		hue: 190,
		countMode: 'working',
		accountId: null,
		paid: true,
		needsApproval: true,
		needsProof: false,
		archived: false,
	},
	{
		id: 'indisponible',
		name: 'Indisponibilité',
		hue: 285,
		countMode: 'working',
		accountId: null,
		paid: false,
		needsApproval: false,
		needsProof: false,
		archived: false,
	},
	{
		id: 'sans-solde',
		name: 'Congé sans solde',
		hue: 60,
		countMode: 'working',
		accountId: null,
		paid: false,
		needsApproval: true,
		needsProof: false,
		archived: false,
	},
]

const ACCOUNTS: readonly LeaveAccount[] = [
	{
		id: 'cp',
		name: 'Congés payés',
		acquisition: { kind: 'monthly', perMonth: 25 / 12 },
		periodStartMonth: 6,
		carryOverMax: 5,
	},
	{
		id: 'rtt',
		name: 'RTT',
		acquisition: { kind: 'annual', days: 10 },
		periodStartMonth: 1,
		carryOverMax: 0,
	},
]

function absence(
	fields: Pick<
		Absence,
		'id' | 'memberId' | 'typeId' | 'startDate' | 'endDate'
	> &
		Partial<Absence>,
): Absence {
	return {
		startPart: 'morning',
		endPart: 'afternoon',
		comment: '',
		proofName: null,
		status: 'approved',
		createdAt: '2026-10-20T09:00:00+02:00',
		createdById: fields.memberId,
		decidedById: 'baptiste',
		decidedAt: '2026-10-21T09:00:00+02:00',
		decisionComment: '',
		...fields,
	}
}

const PENDING = {
	status: 'pending',
	decidedById: null,
	decidedAt: null,
} as const

const ABSENCES: readonly Absence[] = [
	absence({
		id: 'abs-julien-novembre',
		memberId: 'julien',
		typeId: 'cp',
		startDate: '2026-11-05',
		endDate: '2026-11-06',
		comment: 'Déménagement, je serai joignable par téléphone.',
		createdAt: '2026-11-03T18:20:00+01:00',
		...PENDING,
	}),
	absence({
		id: 'abs-sophie-maladie',
		memberId: 'sophie',
		typeId: 'maladie',
		startDate: '2026-11-02',
		endDate: '2026-11-06',
		proofName: 'arret-de-travail-lambert.pdf',
		createdAt: '2026-11-02T07:45:00+01:00',
		decidedById: null,
	}),
	absence({
		id: 'abs-lucie-aprem',
		memberId: 'lucie',
		typeId: 'cp',
		startDate: '2026-11-04',
		endDate: '2026-11-04',
		startPart: 'afternoon',
		comment: 'Rendez-vous médical.',
	}),
	absence({
		id: 'abs-hugo-intemperies',
		memberId: 'hugo',
		typeId: 'intemperies',
		startDate: '2026-11-03',
		endDate: '2026-11-03',
		comment: 'Chantier extérieur arrêté, vent violent.',
		createdById: 'karim',
		decidedById: null,
	}),
	absence({
		id: 'abs-karim-novembre',
		memberId: 'karim',
		typeId: 'cp',
		startDate: '2026-11-09',
		endDate: '2026-11-13',
		createdAt: '2026-10-08T11:00:00+02:00',
	}),
	absence({
		id: 'abs-yanis-cfa',
		memberId: 'yanis',
		typeId: 'formation',
		startDate: '2026-11-16',
		endDate: '2026-11-20',
		comment: 'Formation CACES en centre agréé.',
		createdAt: '2026-09-15T10:00:00+02:00',
	}),
	absence({
		id: 'abs-thomas-rtt',
		memberId: 'thomas',
		typeId: 'rtt',
		startDate: '2026-11-20',
		endDate: '2026-11-20',
	}),
	absence({
		id: 'abs-thomas-rtt-pending',
		memberId: 'thomas',
		typeId: 'rtt',
		startDate: '2026-11-27',
		endDate: '2026-11-27',
		createdAt: '2026-11-03T08:10:00+01:00',
		...PENDING,
	}),
	absence({
		id: 'abs-thomas-refus',
		memberId: 'thomas',
		typeId: 'cp',
		startDate: '2026-11-23',
		endDate: '2026-11-25',
		status: 'rejected',
		decisionComment:
			'Livraison de la rénovation Dupont cette semaine-là, on décale en décembre.',
		createdAt: '2026-10-28T16:30:00+01:00',
		decidedAt: '2026-10-29T08:15:00+01:00',
	}),
	absence({
		id: 'abs-baptiste-noel',
		memberId: 'baptiste',
		typeId: 'cp',
		startDate: '2026-12-24',
		endDate: '2026-12-31',
		createdAt: '2026-09-30T19:00:00+02:00',
	}),
	absence({
		id: 'abs-julien-aout',
		memberId: 'julien',
		typeId: 'cp',
		startDate: '2026-08-10',
		endDate: '2026-08-21',
		createdAt: '2026-06-15T09:00:00+02:00',
	}),
	absence({
		id: 'abs-karim-aout',
		memberId: 'karim',
		typeId: 'cp',
		startDate: '2026-08-17',
		endDate: '2026-08-21',
		createdAt: '2026-06-02T09:00:00+02:00',
	}),
	absence({
		id: 'abs-lucie-juillet',
		memberId: 'lucie',
		typeId: 'cp',
		startDate: '2026-07-27',
		endDate: '2026-07-30',
		createdAt: '2026-06-20T09:00:00+02:00',
	}),
	absence({
		id: 'abs-thomas-aout',
		memberId: 'thomas',
		typeId: 'cp',
		startDate: '2026-08-03',
		endDate: '2026-08-14',
		createdAt: '2026-05-12T09:00:00+02:00',
	}),
	absence({
		id: 'abs-baptiste-rtt',
		memberId: 'baptiste',
		typeId: 'rtt',
		startDate: '2026-10-16',
		endDate: '2026-10-16',
		createdAt: '2026-10-05T09:00:00+02:00',
	}),
	absence({
		id: 'abs-karim-rtt',
		memberId: 'karim',
		typeId: 'rtt',
		startDate: '2026-09-18',
		endDate: '2026-09-18',
		createdAt: '2026-09-02T09:00:00+02:00',
	}),
]

function adjustment(
	fields: Pick<
		BalanceAdjustment,
		'id' | 'memberId' | 'accountId' | 'date' | 'days' | 'kind' | 'reason'
	>,
): BalanceAdjustment {
	return { authorId: 'baptiste', ...fields }
}

const ADJUSTMENTS: readonly BalanceAdjustment[] = [
	adjustment({
		id: 'adj-karim-report',
		memberId: 'karim',
		accountId: 'cp',
		date: '2026-06-01',
		days: 5,
		kind: 'carry_over',
		reason: 'Report des congés 2025-2026',
	}),
	adjustment({
		id: 'adj-julien-report',
		memberId: 'julien',
		accountId: 'cp',
		date: '2026-06-01',
		days: 4,
		kind: 'carry_over',
		reason: 'Report des congés 2025-2026',
	}),
	adjustment({
		id: 'adj-baptiste-report',
		memberId: 'baptiste',
		accountId: 'cp',
		date: '2026-06-01',
		days: 5,
		kind: 'carry_over',
		reason: 'Report des congés 2025-2026',
	}),
	adjustment({
		id: 'adj-lucie-report',
		memberId: 'lucie',
		accountId: 'cp',
		date: '2026-06-01',
		days: 2,
		kind: 'carry_over',
		reason: 'Report des congés 2025-2026',
	}),
	adjustment({
		id: 'adj-lucie-fractionnement',
		memberId: 'lucie',
		accountId: 'cp',
		date: '2026-09-01',
		days: 1,
		kind: 'adjustment',
		reason: 'Jour de fractionnement',
	}),
	adjustment({
		id: 'adj-thomas-chantier',
		memberId: 'thomas',
		accountId: 'rtt',
		date: '2026-10-15',
		days: -0.5,
		kind: 'adjustment',
		reason: 'Demi-journée récupérée sur le chantier Moreau',
	}),
]

function manonPlanning(): DayOverride[] {
	return daysInRange('2026-10-12', '2026-11-20')
		.filter((date) => weekdayOf(date) < 5 && !isHoliday(date))
		.map(
			(date): DayOverride =>
				date === '2026-11-13'
					? {
							memberId: 'manon',
							date,
							kind: 'off',
							start: '',
							end: '',
							note: 'Pas de mission',
						}
					: {
							memberId: 'manon',
							date,
							kind: 'work',
							start: '08:00',
							end: '17:00',
							lunch: DEFAULT_LUNCH,
							note: '',
						},
		)
}

const OVERRIDES: readonly DayOverride[] = [
	...manonPlanning(),
	{
		memberId: 'karim',
		date: '2026-11-07',
		kind: 'work',
		start: '08:00',
		end: '12:00',
		lunch: null,
		note: 'Livraison du matériel sur le chantier Dupont',
	},
	{
		memberId: 'julien',
		date: '2026-11-23',
		kind: 'off',
		start: '',
		end: '',
		note: 'Pont accordé',
	},
]

const SURCHARGES: readonly Surcharge[] = [
	{
		id: 'hs25',
		name: 'Heures supplémentaires',
		rate: 25,
		kind: 'overtime',
		archived: false,
	},
	{
		id: 'hs50',
		name: 'Heures supplémentaires au-delà de 8 h',
		rate: 50,
		kind: 'overtime',
		archived: false,
	},
	{
		id: 'dimanche',
		name: 'Travail du dimanche',
		rate: 100,
		kind: 'sunday',
		archived: false,
	},
	{
		id: 'ferie',
		name: 'Jour férié travaillé',
		rate: 100,
		kind: 'holiday',
		archived: false,
	},
	{
		id: 'nuit',
		name: 'Travail de nuit',
		rate: 25,
		kind: 'night',
		archived: false,
	},
]

function extra(
	fields: Pick<
		ExtraPeriod,
		'id' | 'memberId' | 'date' | 'start' | 'end' | 'reason'
	> &
		Partial<ExtraPeriod>,
): ExtraPeriod {
	return {
		lunch: null,
		surchargeId: 'hs25',
		rate: 25,
		status: 'approved',
		createdAt: '2026-10-30T18:00:00+01:00',
		createdById: fields.memberId,
		decidedById: 'baptiste',
		decidedAt: '2026-10-31T08:30:00+01:00',
		decisionComment: '',
		...fields,
	}
}

const EXTRAS: readonly ExtraPeriod[] = [
	extra({
		id: 'extra-hugo-samedi',
		memberId: 'hugo',
		date: '2026-11-14',
		start: '08:00',
		end: '12:00',
		reason: 'Rattrapage du chantier Faure avant la livraison',
		createdAt: '2026-11-03T17:40:00+01:00',
		status: 'pending',
		decidedById: null,
		decidedAt: null,
	}),
	extra({
		id: 'extra-sophie-ferie',
		memberId: 'sophie',
		date: '2026-11-11',
		start: '08:00',
		end: '12:00',
		reason: 'Astreinte chauffage de la copropriété',
		surchargeId: 'ferie',
		rate: 100,
		createdAt: '2026-11-04T08:10:00+01:00',
		status: 'pending',
		decidedById: null,
		decidedAt: null,
	}),
	extra({
		id: 'extra-julien-soir',
		memberId: 'julien',
		date: '2026-11-03',
		start: '17:00',
		end: '20:00',
		reason: 'Pose du volet roulant chez M. Garnier, finitions',
		createdAt: '2026-11-03T20:10:00+01:00',
		decidedAt: '2026-11-04T08:20:00+01:00',
	}),
	extra({
		id: 'extra-thomas-depannage',
		memberId: 'thomas',
		date: '2026-10-28',
		start: '17:00',
		end: '19:30',
		reason: 'Dépannage urgent d’un tableau électrique',
	}),
	extra({
		id: 'extra-hugo-dimanche',
		memberId: 'hugo',
		date: '2026-10-25',
		start: '08:00',
		end: '14:00',
		reason: 'Terrasse à terminer avant la pluie',
		surchargeId: 'dimanche',
		rate: 100,
	}),
	extra({
		id: 'extra-lucie-cloture',
		memberId: 'lucie',
		date: '2026-11-02',
		start: '17:30',
		end: '19:00',
		reason: 'Clôture mensuelle de la facturation',
		surchargeId: null,
		rate: 0,
	}),
	extra({
		id: 'extra-thomas-samedi',
		memberId: 'thomas',
		date: '2026-10-31',
		start: '08:00',
		end: '12:00',
		reason: 'Avance sur le chantier Moreau',
		status: 'rejected',
		decisionComment: 'Pas de commande validée, on reste sur l’horaire normal.',
	}),
]

export const INITIAL_RH_STATE: RhState = {
	members: MEMBERS,
	absenceTypes: ABSENCE_TYPES,
	accounts: ACCOUNTS,
	absences: ABSENCES,
	adjustments: ADJUSTMENTS,
	overrides: OVERRIDES,
	surcharges: SURCHARGES,
	extras: EXTRAS,
}

export function defaultMember(fields: {
	id: string
	firstName: string
	lastName: string
	workEmail: string
}): Member {
	return member({
		id: fields.id,
		firstName: fields.firstName,
		lastName: fields.lastName,
		jobTitle: '',
		hue: hueOf(fields.id),
		contracts: [
			contract(`c-${fields.id}-1`, { type: 'cdi', startDate: '2024-01-01' }),
		],
		managerId: null,
		workEmail: fields.workEmail,
		mobile: '',
	})
}

function hueOf(id: string): number {
	let hash = 0
	for (const char of id) hash = (hash * 31 + char.charCodeAt(0)) % 360
	return hash
}
