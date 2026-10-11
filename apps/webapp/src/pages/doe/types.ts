import type { Schemas } from '#/api/api.client'

export type Doe = Schemas.DoeResponse
export type DoeSection = Schemas.DoeSection
export type DoeSectionContent = Schemas.DoeSectionContent
export type DoeSupply = Schemas.DoeSupply
export type DoeFile = Schemas.DoeFile
export type DoeParty = Schemas.DoeParty
export type PartyRole = Schemas.PartyRole
export type MakerLabel = Schemas.MakerLabel
export type PhotosPerPage = Schemas.PhotosPerPage
export type SupplierSnapshot = Schemas.SupplierSnapshot
export type UpdateDoeRequest = Schemas.UpdateDoeRequest

export type SectionKind = DoeSectionContent['kind']

export interface PartyForm {
	role: PartyRole
	name: string
	address: string
	phone: string
	email: string
	logoKey: string | null
}

export interface DoeForm {
	title: string
	place: string
	lot: string
	issuedOn: string
	author: string
	revision: string
	coverPhotoKey: string | null
	parties: PartyForm[]
	sections: DoeSection[]
}

export interface NumberedSection {
	section: DoeSection
	number: number | null
}

export const PARTY_ROLES: PartyRole[] = [
	'project_owner',
	'project_manager',
	'contractor',
]

export const PARTY_LABELS: Record<PartyRole, string> = {
	project_owner: 'Maîtrise d’ouvrage',
	project_manager: 'Maîtrise d’œuvre',
	contractor: 'Entreprise',
}

export const KIND_LABELS: Record<SectionKind, string> = {
	supplies: 'Fournitures',
	photos: 'Photos',
	plans: 'Plans',
	blank_pages: 'Pages libres',
}

export const SECTION_KINDS: SectionKind[] = [
	'supplies',
	'photos',
	'plans',
	'blank_pages',
]

export const MAKER_LABEL_TEXT: Record<MakerLabel, string> = {
	manufacturer: 'Fabricant',
	nursery: 'Pépinière',
}

export const PER_PAGE_VALUES: Record<PhotosPerPage, number> = {
	one: 1,
	two: 2,
	four: 4,
}
