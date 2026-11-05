import type { CollectionSource } from '#/pages/automation/lib/collections'

type PresetSpec = {
	source: CollectionSource
	importedAt: string
	document: unknown
}

const ADDRESS_FEATURE = {
	type: 'object',
	required: ['type', 'geometry', 'properties'],
	properties: {
		type: { type: 'string', enum: ['Feature'] },
		geometry: {
			type: 'object',
			properties: {
				type: { type: 'string', enum: ['Point'] },
				coordinates: {
					type: 'array',
					items: { type: 'number' },
					example: [2.3488, 48.8534],
				},
			},
		},
		properties: { $ref: '#/components/schemas/AddressProperties' },
	},
}

const ADDRESS_API = {
	openapi: '3.0.3',
	info: {
		title: 'API Adresse',
		version: '1.0',
		description:
			'Géocodage des adresses françaises à partir de la Base Adresse Nationale. Extrait.',
	},
	servers: [{ url: 'https://api-adresse.data.gouv.fr' }],
	components: {
		schemas: {
			AddressFeatureCollection: {
				type: 'object',
				required: ['type', 'features'],
				properties: {
					type: { type: 'string', enum: ['FeatureCollection'] },
					features: {
						type: 'array',
						items: { $ref: '#/components/schemas/AddressFeature' },
					},
				},
			},
			AddressFeature: ADDRESS_FEATURE,
			AddressProperties: {
				type: 'object',
				required: ['label', 'score', 'type', 'postcode', 'city', 'citycode'],
				properties: {
					label: { type: 'string', example: '12 Rue des Lilas 44000 Nantes' },
					score: { type: 'number', example: 0.97 },
					type: {
						type: 'string',
						enum: ['housenumber', 'street', 'locality', 'municipality'],
					},
					housenumber: { type: 'string', example: '12' },
					street: { type: 'string', example: 'Rue des Lilas' },
					postcode: { type: 'string', example: '44000' },
					city: { type: 'string', example: 'Nantes' },
					citycode: { type: 'string', example: '44109' },
					context: {
						type: 'string',
						example: '44, Loire-Atlantique, Pays de la Loire',
					},
				},
			},
		},
	},
	paths: {
		'/search/': {
			get: {
				operationId: 'searchAddress',
				summary: 'Rechercher une adresse',
				tags: ['Géocodage'],
				parameters: [
					{
						name: 'q',
						in: 'query',
						required: true,
						description: 'Adresse recherchée',
						schema: { type: 'string' },
					},
					{
						name: 'limit',
						in: 'query',
						description: 'Nombre maximum de résultats',
						schema: { type: 'integer', default: 5 },
					},
					{
						name: 'postcode',
						in: 'query',
						description: 'Filtrer par code postal',
						schema: { type: 'string' },
					},
					{
						name: 'type',
						in: 'query',
						description: 'Type de résultat attendu',
						schema: {
							type: 'string',
							enum: ['housenumber', 'street', 'locality', 'municipality'],
						},
					},
				],
				responses: {
					'200': {
						description: 'Adresses trouvées',
						content: {
							'application/json': {
								schema: {
									$ref: '#/components/schemas/AddressFeatureCollection',
								},
							},
						},
					},
				},
			},
		},
		'/reverse/': {
			get: {
				operationId: 'reverseGeocode',
				summary: 'Trouver l’adresse d’un point GPS',
				tags: ['Géocodage'],
				parameters: [
					{
						name: 'lon',
						in: 'query',
						required: true,
						description: 'Longitude',
						schema: { type: 'number' },
					},
					{
						name: 'lat',
						in: 'query',
						required: true,
						description: 'Latitude',
						schema: { type: 'number' },
					},
				],
				responses: {
					'200': {
						description: 'Adresse la plus proche',
						content: {
							'application/json': {
								schema: {
									$ref: '#/components/schemas/AddressFeatureCollection',
								},
							},
						},
					},
				},
			},
		},
	},
}

const COMPANY_API = {
	openapi: '3.0.3',
	info: {
		title: 'Recherche d’entreprises',
		version: '1.0',
		description:
			'Recherche des entreprises françaises par nom, SIREN ou SIRET. Extrait.',
	},
	servers: [{ url: 'https://recherche-entreprises.api.gouv.fr' }],
	components: {
		schemas: {
			SearchResult: {
				type: 'object',
				required: [
					'results',
					'total_results',
					'page',
					'per_page',
					'total_pages',
				],
				properties: {
					results: {
						type: 'array',
						items: { $ref: '#/components/schemas/Company' },
					},
					total_results: { type: 'integer', example: 1 },
					page: { type: 'integer', example: 1 },
					per_page: { type: 'integer', example: 10 },
					total_pages: { type: 'integer', example: 1 },
				},
			},
			Company: {
				type: 'object',
				required: ['siren', 'nom_complet', 'siege'],
				properties: {
					siren: { type: 'string', example: '552100554' },
					nom_complet: { type: 'string', example: 'MENUISERIE DUVAL' },
					activite_principale: { type: 'string', example: '43.32A' },
					nombre_etablissements: { type: 'integer', example: 2 },
					siege: { $ref: '#/components/schemas/Establishment' },
				},
			},
			Establishment: {
				type: 'object',
				required: ['siret'],
				properties: {
					siret: { type: 'string', example: '55210055400013' },
					adresse: { type: 'string', example: '12 RUE DES LILAS 44000 NANTES' },
					code_postal: { type: 'string', example: '44000' },
					libelle_commune: { type: 'string', example: 'NANTES' },
				},
			},
		},
	},
	paths: {
		'/search': {
			get: {
				operationId: 'searchCompanies',
				summary: 'Rechercher une entreprise',
				tags: ['Entreprises'],
				parameters: [
					{
						name: 'q',
						in: 'query',
						required: true,
						description: 'Nom, SIREN ou SIRET',
						schema: { type: 'string' },
					},
					{
						name: 'page',
						in: 'query',
						schema: { type: 'integer', default: 1 },
					},
					{
						name: 'per_page',
						in: 'query',
						schema: { type: 'integer', default: 10 },
					},
				],
				responses: {
					'200': {
						description: 'Entreprises trouvées',
						content: {
							'application/json': {
								schema: { $ref: '#/components/schemas/SearchResult' },
							},
						},
					},
				},
			},
		},
	},
}

export const COLLECTION_PRESETS: readonly PresetSpec[] = [
	{
		source: {
			kind: 'url',
			url: 'https://api-adresse.data.gouv.fr/openapi.json',
		},
		importedAt: '2026-10-28T10:15:00Z',
		document: ADDRESS_API,
	},
	{
		source: { kind: 'file', fileName: 'recherche-entreprises.yaml' },
		importedAt: '2026-11-02T16:40:00Z',
		document: COMPANY_API,
	},
]
